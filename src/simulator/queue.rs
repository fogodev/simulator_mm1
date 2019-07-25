use crate::simulator::client::Client;
use crate::simulator::exponential_time_generator::ExponentialTime;
use crate::simulator::sample_accumulators::sample::Sample;
use crate::simulator::sample_accumulators::stochastic_process_sample::StochasticProcessSample;
use std::collections::{HashMap, VecDeque};
use std::mem::swap;

struct Event {
    name: String,
    timestamp: f64,
}

#[derive(Debug, Copy, Clone)]
pub enum QueuePolicy {
    FCFS,
    LCFS,
}

pub const NQ: &str = "Nq";
pub const N: &str = "N";
pub const W: &str = "W";
pub const X: &str = "X";
pub const T: &str = "T";

const CLIENT_ARRIVAL: &str = "client_arrival";
const END_OF_SERVICE: &str = "end_of_service";

pub struct Queue {
    samples: HashMap<String, Sample>,
    stochastic_process_samples: HashMap<String, StochasticProcessSample>,
    queue_policy: QueuePolicy,
    lambda: f64,
    queue: VecDeque<Client>,
    client_in_service: Option<Client>,
    past_events: Vec<Event>,
    current_time: f64,
    exponential_time_generator: ExponentialTime,
}

impl Queue {
    pub fn new(lambda: f64, queue_policy: QueuePolicy, seed: u64) -> Self {
        let mut exponential_time_generator = ExponentialTime::new(seed);
        let first_arrival_time = exponential_time_generator.get(lambda);
        let mut queue = Self {
            samples: HashMap::new(),
            stochastic_process_samples: HashMap::new(),
            queue_policy,
            lambda,
            queue: VecDeque::new(),
            client_in_service: None,
            past_events: vec![],
            current_time: 0.0,
            exponential_time_generator,
        };
        queue.add_event(CLIENT_ARRIVAL, first_arrival_time);

        queue
    }

    fn initialize_sample_collectors(&mut self, num_samples: usize) {
        let mut samples = HashMap::with_capacity(3);
        samples.insert(W.to_string(), Sample::new(num_samples));
        samples.insert(X.to_string(), Sample::new(num_samples));
        samples.insert(T.to_string(), Sample::new(num_samples));
        self.samples = samples;

        let mut stochastic_process_samples = HashMap::with_capacity(2);
        stochastic_process_samples
            .insert(NQ.to_string(), StochasticProcessSample::new(num_samples));
        stochastic_process_samples.insert(N.to_string(), StochasticProcessSample::new(num_samples));
        self.stochastic_process_samples = stochastic_process_samples;
    }

    fn register_current_state_values(&mut self) {
        let ns = if self.client_in_service.is_some() {
            1
        } else {
            0
        };
        let nq = self.queue.len();
        self.stochastic_process_samples
            .get_mut(N)
            .unwrap()
            .append(self.current_time, ns + nq);
        self.stochastic_process_samples
            .get_mut(NQ)
            .unwrap()
            .append(self.current_time, nq);
    }

    fn register_client_queue_and_server_times(&mut self, client: &Client) {
        let w = client.calculate_event_time(W);
        let x = client.calculate_event_time(X);
        self.samples.get_mut(W).unwrap().append(w);
        self.samples.get_mut(X).unwrap().append(x);
        self.samples.get_mut(T).unwrap().append(w + x);
    }

    fn add_event(&mut self, name: &str, event_duration: f64) {
        self.past_events.push(Event {
            name: name.to_string(),
            timestamp: self.current_time + event_duration,
        })
    }

    fn get_next_event(&mut self) -> Event {
        assert!(
            !self.past_events.is_empty(),
            "A lista de eventos está vazia!"
        );
        let mut smallest_element_index = 0;
        let mut smallest_event_time = std::f64::INFINITY;
        for (index, event) in self.past_events.iter().enumerate() {
            if smallest_event_time > event.timestamp {
                smallest_element_index = index;
                smallest_event_time = event.timestamp;
            }
        }
        self.past_events.swap_remove(smallest_element_index)
    }

    fn get_next_client(&mut self) -> Client {
        assert!(!self.queue.is_empty(), "A fila está vazia!");
        match self.queue_policy {
            QueuePolicy::FCFS => self.queue.pop_back().unwrap(),
            QueuePolicy::LCFS => self.queue.pop_front().unwrap(),
        }
    }

    fn handle_arrival_event(&mut self) {
        let next_client_arrival = self.exponential_time_generator.get(self.lambda);
        self.add_event(CLIENT_ARRIVAL, next_client_arrival);
        let mut client = Client::new(self.exponential_time_generator.get(1.0));
        client.register_start(W, self.current_time);
        if self.queue.is_empty() && self.client_in_service.is_none() {
            client.register_end(W, self.current_time);
            client.register_start(X, self.current_time);
            self.add_event(END_OF_SERVICE, client.x());
            self.client_in_service = Some(client);
        } else {
            self.queue.push_front(client);
            self.register_current_state_values();
        }
    }

    fn end_of_service_event(&mut self) {
        if self.client_in_service.is_some() {
            let mut current_wrapped_client = None;
            swap(&mut current_wrapped_client, &mut self.client_in_service);
            let mut current_client = current_wrapped_client.unwrap();
            current_client.register_end(X, self.current_time);
            self.register_client_queue_and_server_times(&current_client);
            if !self.queue.is_empty() {
                let mut next_client = self.get_next_client();
                next_client.register_end(W, self.current_time);
                next_client.register_start(X, self.current_time);
                self.add_event(END_OF_SERVICE, next_client.x());
                self.client_in_service = Some(next_client);
            } else {
                self.client_in_service = None;
            }
            self.register_current_state_values()
        }
    }

    pub fn run_one_simulation_round(
        &mut self,
        client_count: usize,
    ) -> (
        HashMap<String, Sample>,
        HashMap<String, StochasticProcessSample>,
    ) {
        self.initialize_sample_collectors(client_count);
        self.register_current_state_values();
        let mut client = 0;
        while client < client_count {
            let event = self.get_next_event();
            self.current_time = event.timestamp;
            if CLIENT_ARRIVAL == event.name {
                self.handle_arrival_event();
            } else if END_OF_SERVICE == event.name {
                self.end_of_service_event();
                client += 1;
            } else {
                panic!("Tipo de evento inválido");
            }
        }

        let mut output_samples = HashMap::new();
        let mut output_stochastic_process_samples = HashMap::new();

        swap(&mut output_samples, &mut self.samples);
        swap(
            &mut output_stochastic_process_samples,
            &mut self.stochastic_process_samples,
        );

        (output_samples, output_stochastic_process_samples)
    }
}
