use crate::observation_logger::ObservationLogger;
use crate::termination::Termination;
use std::fmt::Display;
use crate::server::Server;
use std::error::Error;
use crate::request_generator::RequestGenerator;

pub async fn run_servers_benchmarks<T: Termination + Clone + Send + 'static,
    O: ObservationLogger<G::Error> + Default + Display + Send + 'static,
    S: for<'a> Server<'a> + Display,
    G: RequestGenerator + Clone + Send + Sync + 'static>(
    concurrent_numbers: &[usize], termination: T, requests: Vec<(S, Vec<G>)>
) -> Result<(), Box<dyn Error>> {
    for (mut server, generators) in requests {
        for generator in generators {
            println!("Staring benches on {}", server);
            server.start().await?;

            for number in concurrent_numbers.iter().copied() {
                let logs = run_concurrent_benchmark::<_, O, _>(number, termination.clone(), generator.clone()).await?;
                println!("Concurrency level: {}", number);
                println!("Results: {}", logs);
            }

            server.stop().await?;
            println!("Ending benches on {}", server);
        }
    }

    Ok(())
}

async fn run_concurrent_benchmark<T: Termination + Clone + Send + 'static, O: ObservationLogger<G::Error> + Default + Send + 'static, G: RequestGenerator + Clone + Send + Sync + 'static>
(concurrent_number: usize, termination: T, generator: G) -> Result<O, Box<dyn std::error::Error>> {
    let futures = (0..concurrent_number)
        .map(|_| run_single_benchmark_loop(termination.clone(), O::default(), generator.clone()))
        .map(|future| tokio::spawn(future))
        .collect::<Vec<_>>();

    let mut logger = O::default();
    for handle in futures {
        let logs = handle.await?;
        logger = logger.merge(logs);
    }

    Ok(logger)
}

async fn run_single_benchmark_loop<T: Termination + Send, O: ObservationLogger<G::Error> + Send, G: RequestGenerator + Send + Sync>(
    mut termination: T,
    mut observation: O,
    generator: G) -> O {
    termination.loop_started();
    loop {
        termination.iteration_started();
        observation.log_start_of_request();

        let result: Result<_, _> = generator.generate_request().await;
        observation.log_end_of_request(result.err());

        if termination.should_terminate() {
            break
        }

        termination.iteration_ended();
    }
    observation
}