// Módulo com a representação de um freguês
mod client;
// Módulo com a representação de um intervalo de confiança
mod confidence_interval;
// Módulo com o gerador de amostras exponenciais
mod exponential_time_generator;
// Módulo com a fila M/M/1
mod queue;
// Módulo com os acumuladores de amostras de variáveis aleatórias e processos estocásticos
mod sample_accumulators;
// Módulo com a exportação dos dados encontrados para um arquivo .csv para análise posterior
mod statistics_output_files;

// Biblioteca externa que renderiza uma progress bar no terminal conforme as rodadas acontecem
use indicatif::{ProgressBar, ProgressStyle};
// Estrutura de dados HashMap da biblioteca padrão do Rust
use std::collections::HashMap;
// Funcionalidade de temporização da biblioteca padrão
use std::time::{Instant, SystemTime};

// Importando a representação do nosso intervalo de confiança
use confidence_interval::ConfidenceInterval;
// Importando a representação do nossa fila M/M/1 e algumas constantes
use queue::{Queue, N, NQ, T, W, X};
// Importando a representação do nosso acumulador de amostras de variáveis aleatórias
use sample_accumulators::sample::Sample;
// Importando a função que escreve os dados coletados pelo simulador num arquivo .csv
use statistics_output_files::write_csv_file;

// Exportando o enum da nossa política de fila, pra ser usado por quem chamar o simulador
pub(crate) use queue::QueuePolicy;
// Exportando o enum do nosso modo de simulação de fila, pra ser usado por quem chamar o simulador
pub(crate) use queue::QueueMode;

// Função interna que constrói um HashMap para coleta de amostras das métricas N, T e X
fn statistics_hash_map(rounds_count: usize) -> HashMap<String, Sample> {
    let mut statistics = HashMap::new();
    statistics.insert(N.to_string(), Sample::new(rounds_count));
    statistics.insert(T.to_string(), Sample::new(rounds_count));
    statistics.insert(X.to_string(), Sample::new(rounds_count));
    statistics
}

// Função que executa o simulador
pub fn simulator(
    rho: f64,                  // Taxa de utilização do sistema
    round_size: usize,         // Quantidade de fregueses por rodada
    rounds_count: usize,       // Quantidade de rodadas
    queue_policy: QueuePolicy, // Política de atendimento FCFS ou LCFS
    queue_mode: QueueMode,     // Modo de simulação
) {
    // Semente a ser utilizada pelo gerador de amostras exponenciais
    let seed = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Erro ao obter o tempo do sistema")
        .as_secs();

    let now = Instant::now();

    // HashMap para coletar médias amostrais de N, T e X por rodada
    let mut means_statistics = statistics_hash_map(rounds_count);
    // HashMap para coletar variâncias amostrais de N, T e X por rodada
    let mut variances_statistics = statistics_hash_map(rounds_count);

    // Acumulador de médias amostrais de W
    let mut w_mean_statistics = Sample::new(rounds_count);
    // Acumulador de variâncias amostrais de W
    let mut w_variance_statistics = Sample::new(rounds_count);
    // Acumulador de médias amostrais de Nq
    let mut nq_mean_statistics = Sample::new(rounds_count);
    // Acumulador de variâncias amostrais de Nq
    let mut nq_variance_statistics = Sample::new(rounds_count);

    // Objeto que representa nossa fila M/M/1
    let mut queue = if queue_mode == QueueMode::ForReal {
        Queue::new(rho, queue_policy, seed)
    } else {
        Queue::check_correctness(queue_policy)
    };

    // Executando a fase transiente
    let transient_phase_size = if queue_mode == QueueMode::ForReal {
        queue.transient_phase()
    } else {0};
    if queue_mode == QueueMode::ForReal {
        println!(
            "\nTotal de fregueses = {}; Política = {:?}; ρ = {}; Tamanho da fase transiente = {}\n",
            round_size, queue_policy, rho, transient_phase_size
        );
    } else {
        println!(
            "\nSimulação para aferição de Corretude do Simulador!\
            \nTotal de fregueses = {}; Política = {:?};",
            round_size, queue_policy
        );
    }

    // Instanciando a barra de progresso que informa o andamento das rodadas de simulação
    let progress_bar = ProgressBar::new(rounds_count as u64);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.white} [{elapsed_precise}] [{bar:40.red/green}] {percent:>3}% {pos:>4}/{len} ({eta_precise})")
            .progress_chars("🔥💧"),
    );
    // For que executa as rodadas da simulação
    for _ in 0..rounds_count {
        progress_bar.inc(1); // Incremento da barra de progresso
                             // Executa uma rodada da simulação, retornando
        let (samples, stochastic_process_samples) = queue.run_one_simulation_round(round_size);
        // Coleta as médias e variâncias amostrais de W, X e T
        for (name, sample) in samples {
            if W == name {
                w_mean_statistics.append(sample.mean());
                w_variance_statistics.append(sample.variance());
            } else {
                means_statistics
                    .get_mut(&name)
                    .unwrap()
                    .append(sample.mean());
                variances_statistics
                    .get_mut(&name)
                    .unwrap()
                    .append(sample.variance());
            }
        }
        // Coleta as médias e variâncias amostrais de N e Nq
        for (name, sample) in stochastic_process_samples {
            if NQ == name {
                nq_mean_statistics.append(sample.mean());
                nq_variance_statistics.append(sample.variance());
            } else {
                means_statistics
                    .get_mut(&name)
                    .unwrap()
                    .append(sample.mean());
                variances_statistics
                    .get_mut(&name)
                    .unwrap()
                    .append(sample.variance());
            }
        }
    }
    progress_bar.finish_with_message("Finalizado"); // Finaliza a barra de progresso

    // Média das médias amostrais de cada rodada de N, T e X
    let means_n_t_x = [
        means_statistics["N"].mean(),
        means_statistics["T"].mean(),
        means_statistics["X"].mean(),
    ];
    println!(
        "Sample Means:\n\tE[N] = {:0.5}\tE[T] = {:0.5}\tE[X] = {:0.5}",
        means_n_t_x[0], means_n_t_x[1], means_n_t_x[2],
    );

    // Média das variâncias amostrais de cada rodada de N, T e X
    let variances_n_t_x = [
        variances_statistics["N"].variance(),
        variances_statistics["T"].variance(),
        variances_statistics["X"].variance(),
    ];
    println!(
        "Sample Variances:\n\tV(N) = {:0.5}\tV(T) = {:0.5}\tV(X) = {:0.5}\n",
        variances_n_t_x[0], variances_n_t_x[1], variances_n_t_x[2],
    );

    // Item a) do relatório
    let mean_w_ci = w_mean_statistics.t_student_95percent();
    let mean_and_ic_w = extract_statistics_and_ci_slice(w_mean_statistics.mean(), &mean_w_ci);
    println!(
        "Sample Mean and Confidence Interval:\n\tE[W]  = {:0.5}\n\t\tIC T-Student:\tL(0.05) = {:0.5};\
         \tCenter = {:0.5}; \tU(0.05) = {:0.5}; \tPrecision = {:0.5}%",
        mean_and_ic_w[0],
        mean_and_ic_w[1],
        mean_and_ic_w[2],
        mean_and_ic_w[3],
        100.0 * mean_and_ic_w[4],
    );

    // Item b) do relatório
    let w_variance = w_variance_statistics.mean();
    let ts_ci_w = w_variance_statistics.t_student_95percent();
    let c2_ci_w = w_variance_statistics.chi_square_95percent(w_variance);
    let variance_and_ic_t_student_chi_square_w =
        extract_statistics_and_2cis_slice(w_variance, &ts_ci_w, &c2_ci_w);
    println!(
        "Sample Variance and Confidence Interval:\n\tV(W)  = {:0.5}\n\t\tIC T-Student:\
         \tL(0.05) = {:0.5};\tCenter = {:0.5};\tU(0.05) = {:0.5};\tPrecision = {:0.5}%\
         \n\t\tIC Chi-Square:\tL(0.05) = {:0.5};\tCenter = {:0.5};\tU(0.05) = {:0.5};\
         \tPrecision = {:0.5}%",
        variance_and_ic_t_student_chi_square_w[0],
        variance_and_ic_t_student_chi_square_w[1],
        variance_and_ic_t_student_chi_square_w[2],
        variance_and_ic_t_student_chi_square_w[3],
        100.0 * variance_and_ic_t_student_chi_square_w[4],
        variance_and_ic_t_student_chi_square_w[5],
        variance_and_ic_t_student_chi_square_w[6],
        variance_and_ic_t_student_chi_square_w[7],
        100.0 * variance_and_ic_t_student_chi_square_w[8],
    );

    // Item c) do relatório
    let mean_nq_ci = nq_mean_statistics.t_student_95percent();
    let mean_and_ic_nq = extract_statistics_and_ci_slice(nq_mean_statistics.mean(), &mean_nq_ci);
    println!(
        "Sample Mean and Confidence Interval:\n\tE[Nq] = {:0.5}\n\t\tIC T-Student:\tL(0.05) = {:0.5};\
         \tCenter = {:0.5}; \tU(0.05) = {:0.5}; \tPrecision = {:0.5}%",
        mean_and_ic_nq[0],
        mean_and_ic_nq[1],
        mean_and_ic_nq[2],
        mean_and_ic_nq[3],
        100.0 * mean_and_ic_nq[4],
    );

    // Item d) do relatório
    let nq_variance = nq_variance_statistics.mean();
    let ts_ci_nq = nq_variance_statistics.t_student_95percent();
    let c2_ci_nq = nq_variance_statistics.chi_square_95percent(nq_variance);
    let variance_and_ic_t_student_chi_square_nq =
        extract_statistics_and_2cis_slice(nq_variance, &ts_ci_nq, &c2_ci_nq);
    println!(
        "Sample Variance and Confidence Interval:\n\tV(Nq) = {:0.5}\n\t\tIC T-Student:\
         \tL(0.05) = {:0.5};\tCenter = {:0.5};\tU(0.05) = {:0.5};\tPrecision = {:0.5}%\
         \n\t\tIC Chi-Square:\tL(0.05) = {:0.5};\tCenter = {:0.5};\tU(0.05) = {:0.5};\
         \tPrecision = {:0.5}%",
        variance_and_ic_t_student_chi_square_nq[0],
        variance_and_ic_t_student_chi_square_nq[1],
        variance_and_ic_t_student_chi_square_nq[2],
        variance_and_ic_t_student_chi_square_nq[3],
        100.0 * variance_and_ic_t_student_chi_square_nq[4],
        variance_and_ic_t_student_chi_square_nq[5],
        variance_and_ic_t_student_chi_square_nq[6],
        variance_and_ic_t_student_chi_square_nq[7],
        100.0 * variance_and_ic_t_student_chi_square_nq[8],
    );

    // Calculando valores analíticos para E[W], V(W), E[Nq], V(Nq)
    let analytic_mean_w = if queue_mode == QueueMode::ForReal {
        rho / (1.0 - rho)
    } else {
        // Cálculo da esperança pela definição, ignoramos o valor de 0 * (1 / 4)
        1.0 * (1.0 / 4.0) + 3.0 * (1.0 / 4.0) + 2.0 * (1.0 / 4.0)
    };
    let analytic_variance_w = match queue_policy {
        QueuePolicy::FCFS => if queue_mode == QueueMode::ForReal {
            (2.0 * rho - rho.powi(2)) / ((1.0 - rho) * (1.0 - rho))
        } else {
            1.0 * (1.0 / 4.0) + (2.0 * 2.0) * (1.0 / 4.0) + (3.0 * 3.0) * (1.0 / 4.0) - analytic_mean_w.powi(2)
        },
        QueuePolicy::LCFS => if queue_mode == QueueMode::ForReal {
            (2.0 * rho - rho.powi(2) + rho.powi(3)) / ((1.0 - rho).powi(3))
        } else {
            1.0 * (1.0 / 4.0) + (5.0 * 5.0) * (1.0 / 4.0) - analytic_mean_w.powi(2)
        }
    };
    let analytic_mean_nq = if queue_mode == QueueMode::ForReal {
        rho.powi(2) / (1.0 - rho)
    } else {
        // Cálculo da esperança pela definição ignoramos o 0 * (4 / 9)
        1.0 * (4.0 / 9.0) + 2.0 * (1.0 / 9.0)
    };
    let analytic_variance_nq = if queue_mode == QueueMode::ForReal {
        (rho.powi(2) + rho.powi(3) - rho.powi(4)) / (1.0 - rho).powi(2)
    } else {
        // Segundo momento - quadrado da média, ignorando o 0 * (4 / 9)
        (1.0 * (4.0 / 9.0) + (2.0 * 2.0) * (1.0 / 9.0)) - analytic_mean_nq.powi(2)
    };
    println!(
        "Analytical values:\n\tE[W]  = {:0.5}\n\tV(W)  = {:0.5}\n\tE[Nq] = {:0.5}\n\tV(Nq) = {:0.5}",
        analytic_mean_w, analytic_variance_w, analytic_mean_nq, analytic_variance_nq
    );

    // Escreve os dados num arquivo .csv
    write_csv_file(
        rho,
        round_size,
        transient_phase_size,
        queue_policy,
        &means_n_t_x,
        &variances_n_t_x,
        &mean_and_ic_w,
        &variance_and_ic_t_student_chi_square_w,
        &mean_and_ic_nq,
        &variance_and_ic_t_student_chi_square_nq,
        analytic_mean_w,
        analytic_variance_w,
        analytic_mean_nq,
        analytic_variance_nq,
        now.elapsed().as_millis() as f64 / 1000.0,
    );

    let mut not_enough = false;

    if !mean_w_ci.value_is_inside(analytic_mean_w) {
        println!("O valor analítico de E[W] não está dentro do IC como esperado");
        not_enough = true;
    }

    if !(ts_ci_w.value_is_inside(analytic_variance_w)
        && c2_ci_w.value_is_inside(analytic_variance_w))
    {
        println!("O valor analítico de V(W) não está dentro do IC como esperado");
        not_enough = true;
    }

    if !mean_nq_ci.value_is_inside(analytic_mean_nq) {
        println!("O valor analítico de E[Nq] não está dentro do IC como esperado");
        not_enough = true;
    }

    if !(ts_ci_nq.value_is_inside(analytic_variance_nq)
        && c2_ci_nq.value_is_inside(analytic_variance_nq))
    {
        println!("O valor analítico de V(Nq) não está dentro do IC como esperado");
        not_enough = true;
    }

    // Caso não tenhamos precisão suficiente, executamos de novo para mais fregueses
    if mean_w_ci.precision() > 0.05 {
        println!(
            "Precisão do IC de E[W] = {:0.5}% não é suficiente",
            100.0 * mean_w_ci.precision(),
        );
        not_enough = true;
    }

    // Caso não tenhamos precisão suficiente, executamos de novo para mais fregueses
    if ts_ci_w.precision() > 0.05 {
        println!(
            "Precisão do IC pela T-Student de V(W) = {:0.5}% não é suficiente",
            100.0 * ts_ci_w.precision(),
        );
        not_enough = true;
    }

    // Caso não tenhamos precisão suficiente, executamos de novo para mais fregueses
    if mean_nq_ci.precision() > 0.05 {
        println!(
            "Precisão do IC de E[Nq] = {:0.5}% não é suficiente",
            100.0 * mean_nq_ci.precision(),
        );
        not_enough = true;
    }

    // Caso não tenhamos precisão suficiente, executamos de novo para mais fregueses
    if ts_ci_nq.precision() > 0.05 {
        println!(
            "Precisão do IC pela T-Student de V(Nq) = {:0.5}% não é suficiente",
            100.0 * ts_ci_nq.precision(),
        );
        not_enough = true;
    }

    // Caso não tenhamos convergência dos ICs, executamos de novo para mais fregueses
    if !ConfidenceInterval::check_convergence(c2_ci_w, ts_ci_w) {
        println!("Os intervalos de confiança para V(W) com T-Student e Chi-Square não convergem");
        not_enough = true;
    }

    // Caso não tenhamos convergência dos ICs, executamos de novo para mais fregueses
    if !ConfidenceInterval::check_convergence(c2_ci_nq, ts_ci_nq) {
        println!(
            "Os intervalos de confiança para V(Nq) com T-Student e Chi-Square não convergem"
        );
        not_enough = true;
    }
    if queue_mode == QueueMode::CheckCorrectness {
        not_enough = false;
    }

    if not_enough {
        println!("Rodando agora para {} clientes", round_size + 100);
        return simulator(rho, round_size + 100, rounds_count, queue_policy, queue_mode);
    }
}

// Função interna para extrair um array contendo a estatística em questão e seu IC
fn extract_statistics_and_ci_slice(statistic: f64, ci: &ConfidenceInterval) -> [f64; 5] {
    [
        statistic,
        ci.lower_bound(),
        ci.center(),
        ci.upper_bound(),
        ci.precision(),
    ]
}

// Função interna para extrair um array contendo a estatística em questão e seus dois ICs
fn extract_statistics_and_2cis_slice(
    statistic: f64,
    first_ci: &ConfidenceInterval,
    second_ci: &ConfidenceInterval,
) -> [f64; 9] {
    [
        statistic,
        first_ci.lower_bound(),
        first_ci.center(),
        first_ci.upper_bound(),
        first_ci.precision(),
        second_ci.lower_bound(),
        second_ci.center(),
        second_ci.upper_bound(),
        second_ci.precision(),
    ]
}
