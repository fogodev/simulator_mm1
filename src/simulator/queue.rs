// Importando várias das nossas construções
use crate::simulator::client::Client;
use crate::simulator::exponential_time_generator::ExponentialTime;
use crate::simulator::sample_accumulators::sample::Sample;
use crate::simulator::sample_accumulators::stochastic_process_sample::StochasticProcessSample;

// Estruturas HashMap e VecDeque (vetor que podemos adicionar e remover no começo e no fim em O(1)
use std::collections::{HashMap, VecDeque};
// Troca dois valores de lugar na memória, utilizado para lidar com o Borrow Checker do Rust
use std::mem::swap;

// Struct para representar um evento, que possui um nome,
// um momento de quando ele aconteceu e sua duração
struct Event {
    name: String,
    birth_time: f64,
    duration: f64,
}

// Enum para representar a política de atendimento da fila, o derive é uma anotação que
// faz o compilador dar algumas características para o enum, Debug permite que o mesmo possa ser
// impresso num println, copy e clone permitem que o mesmo possa ser copiado de um lugar para outro
#[derive(Debug, Copy, Clone)]
pub enum QueuePolicy {
    FCFS,
    LCFS,
}

// Enum para determinar em qual modo estamos rodando o simulador, no modo para valer ou no
// modo de verificar a corretude do mesmo, com chegadas e tempo de serviço deterministicoPra
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum QueueMode {
    ForReal,
    CheckCorrectness
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
    color: usize,                                // Cor da fila na rodada atual
    mode: QueueMode,                              // O modo de funcionamento da fila
}

impl Queue {
    // Instancia uma nova fila, de acordo com o lambda, política de atendimento e semente
    pub fn new(lambda: f64, queue_policy: QueuePolicy, seed: u64) -> Self {
        // Instancia o gerador de amostras exponenciais
        let mut exponential_time_generator = ExponentialTime::new(seed);
        // Calcula quando será o primeiro evento de chegada
        let first_event_duration = exponential_time_generator.get(lambda);
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
            color: 0,
            mode: QueueMode::ForReal,
        };
        // Adiciona o evento da primeira chegada
        queue.add_event(CLIENT_ARRIVAL, first_event_duration);
        queue // Retorna a fila instanciada
    }

    pub fn check_correctness(queue_policy: QueuePolicy) -> Self {
        Self {
            samples: HashMap::new(),
            stochastic_process_samples: HashMap::new(),
            queue_policy,
            lambda: 0.0,
            queue: VecDeque::new(),
            client_in_service: None,
            past_events: vec![],
            current_time: 0.0,
            exponential_time_generator: ExponentialTime::new(0), // Não é usado
            color: 0,
            mode: QueueMode::CheckCorrectness,
        }
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
    fn add_event(&mut self, name: &str, duration: f64) {
        let current_time = self.current_time;
        self.past_events.push(Event {
            name: name.to_string(),
            birth_time: current_time,
            duration,
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
            let event_time = event.birth_time + event.duration;
            if smallest_event_time > event_time {
                smallest_element_index = index;
                smallest_event_time = event_time;
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
        // Instancia um novo freguês para entrar na fila ou ser atendido
        let mut client = if self.mode == QueueMode::ForReal {
            // Calcula o evento da próxima chegada
            let next_client_arrival_duration = self.exponential_time_generator.get(self.lambda);
            // Adiciona o evento da próxima chegada na lista de eventos caso seja uma simulação real
            self.add_event(CLIENT_ARRIVAL, next_client_arrival_duration);
            Client::new(self.exponential_time_generator.get(1.0), self.color)
        } else {
            Client::new(0.0, self.color)
        };
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
            if self.mode == QueueMode::ForReal {
                self.add_event(END_OF_SERVICE, client.x());
            }
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
            // Coleta as métricas W, X e T desse freguês se ele for da cor rodada atual
            if current_client.color() == self.color {
                self.register_client_queue_and_server_times(&current_client);
            }
            if !self.queue.is_empty() {
                // Caso a fila não esteja vazia
                // Seleciona o próximo freguês
                let mut next_client = self.get_next_client();
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
        }
        self.register_current_state_values(); // Registra o estado atual da fila
    }

    pub fn transient_phase(&mut self) -> usize {
        // Coletores de métricas com um valor qualquer, essas métricas serão descartadas
        self.initialize_sample_collectors(5000);
        self.register_current_state_values();
        // Contabilizamos o período ocupado
        let mut busy_time = 0.0;
        // Contabilizamos o tamanho da fase transiente
        let mut transient_phase_counter = 0;
        let mut stable_queue_counter = 0usize;
        loop {
            // Acumulamos os períodos ocupados
            busy_time += self.handle_transient_phase_events();
            // Incrementamos o tamanho atual da fase transiente
            transient_phase_counter += 1;
            // Calculamos um rho simulado, que é taxa atual de utilização da fila
            let simulated_rho = busy_time / self.current_time;
            if 1.0 - f64::min(simulated_rho, self.lambda) / f64::max(simulated_rho, self.lambda)
                <= 0.01
            {
                // Se o rho da simulação estiver razoavelmente próximo do rho contabilizamos
                stable_queue_counter += 1;
            } else {
                // Se não estiver próximo o suficiente, resetamos a contagem para zero
                stable_queue_counter = 0;
            }
            if stable_queue_counter == 500 {
                // Se atingimos 500 iterações sequenciais com estabilidade, então podemos sair
                // da fase transiente
                break transient_phase_counter;
            }
        }
    }

    fn handle_transient_phase_events(&mut self) -> f64 {
        // Selecionamos o próximo evento
        let event = self.get_next_event();
        // Aqui vemos se o evento atual está ocorrendo ou não durante um período ocupado
        let new_busy_time = if !self.queue.is_empty() || self.client_in_service.is_some() {
            event.birth_time + event.duration - self.current_time
        } else {
            0.0
        };
        self.current_time = event.birth_time + event.duration; // Atualizamos o tempo atual da fila

        if CLIENT_ARRIVAL == event.name {
            // Caso seja evento de chegada de freguês
            self.handle_arrival_event(); // Processamos a chegada
        } else if END_OF_SERVICE == event.name {
            // Caso seja evento de fim de serviço
            self.end_of_service_event(); // Processamos a saída
        } else {
            panic!("Tipo de evento inválido"); // Apenas eventos de chegada e saída são válidos
        }
        new_busy_time
    }

    // Executa uma rodada de simulação da fila e retorna as amostras coletadas das métricas
    pub fn run_one_simulation_round(
        &mut self,
        client_count: usize, // Número de freguêses dessa rodada
    ) -> (
        HashMap<String, Sample>,
        HashMap<String, StochasticProcessSample>,
    ) {
        // Atualiza a cor da fila preparando a mesma para a rodada de simulação
        self.color += 1;
        // Inicializa os coletores de amostras
        self.initialize_sample_collectors(client_count);
        self.register_current_state_values(); // Registra o estado atual da fila
        let mut client = 0;
       if self.mode == QueueMode::ForReal {
           while client < client_count {
               // Enquanto não processarmos todos os clientes pedidos
               let event = self.get_next_event(); // Pegamos o próximo evento
               self.current_time = event.birth_time + event.duration; // Atualizamos o tempo atual da fila
               if CLIENT_ARRIVAL == event.name {
                   // Caso seja evento de chegada de freguês
                   self.handle_arrival_event(); // Processamos a chegada
               } else if END_OF_SERVICE == event.name {
                   // Caso seja evento de fim de serviço
                   if let Some(current_client) = &self.client_in_service {
                       if current_client.color() == self.color {
                           // Contabilizamos o cliente satisfeito nessa rodada caso seja da cor atual
                           client += 1;
                       }
                   }
                   self.end_of_service_event(); // Processamos a saída
               } else {
                   panic!("Tipo de evento inválido"); // Apenas eventos de chegada e saída são válidos
               }
           }
       } else {
           // Aqui forçamos uma fila onde temos chegadas nos momentos 0, 1, 2 e 3, com tempo
           // de serviço constante igual a 2, de maneira que assim temos um resultado deterministico
           // temos um ciclo de 9 segundos de duração
            while client < client_count {
                for _step in 0..9 {
                    match _step {
                        0 => self.handle_arrival_event(),
                        1 => self.handle_arrival_event(),
                        2 => {
                            self.handle_arrival_event();
                            self.end_of_service_event();
                        },
                        3 => self.handle_arrival_event(),
                        4 => self.end_of_service_event(),
                        6 => self.end_of_service_event(),
                        8 => self.end_of_service_event(),
                        _ => ()
                    }
                    self.current_time += 1.0;
                }
                client += 4; // Tendo em vista que 4 clientes chegaram e saíram em cada ciclo
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
