// Importando várias das nossas construções
use crate::simulator::client::Client;
use crate::simulator::exponential_time_generator::ExponentialTime;
use crate::simulator::sample_accumulators::sample::Sample;
use crate::simulator::sample_accumulators::stochastic_process_sample::StochasticProcessSample;

// Estruturas HashMap e VecDeque (vetor que podemos adicionar e remover no começo e no fim em O(1)
use std::collections::{HashMap, VecDeque};
// Troca dois valores de lugar na memória, utilizado para lidar com o Borrow Checker do Rust
use std::mem::swap;

// Struct para representar um evento, que possui um nome e um tempo de quando aconteceu
struct Event {
    name: String,
    timestamp: f64,
}

// Enum para representar a política de atendimento da fila, o derive é uma anotação que
// faz o compilador dar algumas características para o enum, Debug permite que o mesmo possa ser
// impresso num println, copy e clone permitem que o mesmo possa ser copiado de um lugar para outro
#[derive(Debug, Copy, Clone)]
pub enum QueuePolicy {
    FCFS,
    LCFS,
}

// Constantes das métricas de interesse
pub const NQ: &str = "Nq";
pub const N: &str = "N";
pub const W: &str = "W";
pub const X: &str = "X";
pub const T: &str = "T";

// Constantes dos tipos de evento que estamos interessados
const CLIENT_ARRIVAL: &str = "client_arrival";
const END_OF_SERVICE: &str = "end_of_service";

// Struct que representa nossa fila M/M/1
pub struct Queue {
    samples: HashMap<String, Sample>, // Acumulador de amostras de variáveis aleatórias
    // Acumulador de amostras de processos estocásticos
    stochastic_process_samples: HashMap<String, StochasticProcessSample>,
    queue_policy: QueuePolicy, // Política de atendimento
    lambda: f64,               // Lambda da fila
    queue: VecDeque<Client>,   // Estrutura que vai representar os clientes na fila
    // Cliente em atendimento no momento, caso haja algum cliente para ser atendido
    client_in_service: Option<Client>,
    past_events: Vec<Event>,                     // Eventos que ocorreram
    current_time: f64,                           // Tempo atual da fila
    exponential_time_generator: ExponentialTime, // Gerador de amostras exponenciais
}

impl Queue {
    // Instancia uma nova fila, de acordo com o lambda, política de atendimento e semente
    pub fn new(lambda: f64, queue_policy: QueuePolicy, seed: u64) -> Self {
        // Instancia o gerador de amostras exponenciais
        let mut exponential_time_generator = ExponentialTime::new(seed);
        // Calcula quando será o primeiro evento de chegada
        let first_arrival_time = exponential_time_generator.get(lambda);
        let mut queue = Self {
            // Instancia a fila
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
        // Adiciona o evento da primeira chegada
        queue.add_event(CLIENT_ARRIVAL, first_arrival_time);
        queue // Retorna a fila instanciada
    }

    // Inicializa os coletores de amostras das métricas de interesse
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

    // Registra as quantidades atuais de N e Nq nos coletores de amostras de processos estocásticos
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

    // Registra as métricas de W, X e T do freguês nos coletores de amostras de variáveis aleatórias
    fn register_client_queue_and_server_times(&mut self, client: &Client) {
        let w = client.calculate_event_time(W);
        let x = client.calculate_event_time(X);
        self.samples.get_mut(W).unwrap().append(w);
        self.samples.get_mut(X).unwrap().append(x);
        self.samples.get_mut(T).unwrap().append(w + x);
    }

    // Adiciona um novo evento na lista de eventos ocorridos
    fn add_event(&mut self, name: &str, event_duration: f64) {
        self.past_events.push(Event {
            name: name.to_string(),
            timestamp: self.current_time + event_duration,
        })
    }

    // Seleciona o próximo evento e remove ele da lista de eventos ocorridos
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

    // Seleciona o próximo cliente a ser atendido, de acordo com a política de atendimento atual
    fn get_next_client(&mut self) -> Client {
        assert!(!self.queue.is_empty(), "A fila está vazia!");
        match self.queue_policy {
            QueuePolicy::FCFS => self.queue.pop_back().unwrap(),
            QueuePolicy::LCFS => self.queue.pop_front().unwrap(),
        }
    }

    // Processa um evento de chegada de freguês
    fn handle_arrival_event(&mut self) {
        // Calcula o evento da próxima chegada
        let next_client_arrival = self.exponential_time_generator.get(self.lambda);
        // Adiciona o evento da próxima chegada na lista de eventos
        self.add_event(CLIENT_ARRIVAL, next_client_arrival);
        // Instancia um novo freguês para entrar na fila ou ser atendido
        let mut client = Client::new(self.exponential_time_generator.get(1.0));
        // Marca o inicio da espera desse freguês
        client.register_start(W, self.current_time);
        // Verifica se a fila está vazia e se não tem nenhum cliente em serviço
        if self.queue.is_empty() && self.client_in_service.is_none() {
            // Como não tem ninguém na fila e nem em serviço, esse freguês entra em atendimento
            // Finalizamos o tempo de espera dele, que como não esperou nada, vale 0
            client.register_end(W, self.current_time);
            // Inicializa o período do atendimento desse freguês
            client.register_start(X, self.current_time);
            // Adiciona o evento do fim de serviço desse freguês de acordo com seu X
            self.add_event(END_OF_SERVICE, client.x());
            self.client_in_service = Some(client); // Colocamos esse freguês em atendimento
        } else {
            // Caso haja alguém na fila ou alguém sendo atendido, freguês vai pra fila de espera
            self.queue.push_front(client);
            self.register_current_state_values(); // Registra o estado atual da fila
        }
    }

    // Processa um evento de fim de atendimento de um freguês
    fn end_of_service_event(&mut self) {
        if self.client_in_service.is_some() {
            // Verifica se há algum freguês sendo atendido
            // Retira esse freguês do atendimento para coletarmos suas métricas
            let mut current_wrapped_client = None;
            swap(&mut current_wrapped_client, &mut self.client_in_service);
            let mut current_client = current_wrapped_client.unwrap();
            // Registra o fim de atendimento desse freguês
            current_client.register_end(X, self.current_time);
            // Coleta as métricas W, X e T desse freguês
            self.register_client_queue_and_server_times(&current_client);
            if !self.queue.is_empty() {
                // Caso a fila não esteja vazia
                let mut next_client = self.get_next_client(); // Seleciona o próximo freguês
                                                              // Finalizamos seu tempo de espera
                next_client.register_end(W, self.current_time);
                // Inicializamos seu tempo de atendimento
                next_client.register_start(X, self.current_time);
                // Registramos o evento de fim de serviço desse freguês
                self.add_event(END_OF_SERVICE, next_client.x());
                self.client_in_service = Some(next_client); // Colocamos esse freguês em atendimento
            } else {
                // Caso a fila esteja vazia, não há cliente para ficar em serviço
                self.client_in_service = None;
            }
            self.register_current_state_values(); // Registra o estado atual da fila
        }
    }

    // Executa uma rodada de simulação da fila e retorna as amostras coletadas das métricas
    pub fn run_one_simulation_round(
        &mut self,
        client_count: usize, // Número de freguêses dessa rodada
    ) -> (
        HashMap<String, Sample>,
        HashMap<String, StochasticProcessSample>,
    ) {
        // Inicializa os coletores de amostras
        self.initialize_sample_collectors(client_count);
        self.register_current_state_values(); // Registra o estado atual da fila
        let mut client = 0;
        while client < client_count {
            // Enquanto não processarmos todos os clientes pedidos
            let event = self.get_next_event(); // Pegamos o próximo evento
            self.current_time = event.timestamp; // Atualizamos o tempo atual da fila
            if CLIENT_ARRIVAL == event.name {
                // Caso seja evento de chegada de freguês
                self.handle_arrival_event(); // Processamos a chegada
            } else if END_OF_SERVICE == event.name {
                // Caso seja evento de fim de serviço
                self.end_of_service_event(); // Processamos a saída
                client += 1; // Contabilizamos o cliente satisfeito nessa rodada
            } else {
                panic!("Tipo de evento inválido"); // Apenas eventos de chegada e saída são válidos
            }
        }
        // Estratégia abaixo é usada para remover e retornar os coletores de amostras da struct
        // sem a necessidade de copiar seus dados, por questões de performance
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
