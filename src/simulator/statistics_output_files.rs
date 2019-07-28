use crate::simulator::QueuePolicy;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

// Função que recebe inúmeros dados e escreve de maneira organizada num arquivo .csv
pub fn write_csv_file(
    rho: f64,
    clients: usize,
    transient_phase: usize,
    policy: QueuePolicy,
    means_n_t_x: &[f64; 3],
    variances_n_t_x: &[f64; 3],
    mean_and_ic_w: &[f64; 5],
    variance_and_ic_t_student_chi_square_w: &[f64; 9],
    mean_and_ic_nq: &[f64; 5],
    variance_and_ic_t_student_chi_square_nq: &[f64; 9],
) {
    let csv_file_path = Path::new("output.csv"); // Path do arquivo csv
    let mut file = if csv_file_path.exists() {
        // Caso o arquivo já exista, abrimos o mesmo em modo append para inserir os dados ao final
        OpenOptions::new()
            .append(true)
            .open(csv_file_path)
            .expect("Unable to open csv file")
    } else {
        // Caso não existe, criamos um novo arquivo e colocamos o cabeçalho das colunas nele
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(csv_file_path)
            .expect("Unable to open csv file");
        file.write_all(
            [
                "rho",
                "fregueses",
                "fase_transiente",
                "E[N]",
                "E[T]",
                "E[X]",
                "V(N)",
                "V(T)",
                "V(X)",
                "E[W]",
                "E[W]_IC_TS_L",
                "E[W]_IC_TS_C",
                "E[W]_IC_TS_U",
                "E[W]_IC_TS_P",
                "V(W)",
                "V(W)_IC_TS_L",
                "V(W)_IC_TS_C",
                "V(W)_IC_TS_U",
                "V(W)_IC_TS_P",
                "V(W)_IC_C2_L",
                "V(W)_IC_C2_C",
                "V(W)_IC_C2_U",
                "V(W)_IC_C2_P",
                "E[Nq]",
                "E[Nq]_IC_TS_L",
                "E[Nq]_IC_TS_C",
                "E[Nq]_IC_TS_U",
                "E[Nq]_IC_TS_P",
                "V(Nq)",
                "V(Nq)_IC_TS_L",
                "V(Nq)_IC_TS_C",
                "V(Nq)_IC_TS_U",
                "V(Nq)_IC_TS_P",
                "V(Nq)_IC_C2_L",
                "V(Nq)_IC_C2_C",
                "V(Nq)_IC_C2_U",
                "V(Nq)_IC_C2_P",
                "policy\n",
            ]
            .join(",")
            .as_bytes(),
        )
        .expect("Failed to write csv file");
        file
    };

    // Adicionamos os dados nas linhas do csv
    let mut output_string = format!("{},{},{},", rho, clients, transient_phase);
    for num in means_n_t_x
        .iter()
        .chain(variances_n_t_x.iter())
        .chain(mean_and_ic_w.iter())
        .chain(variance_and_ic_t_student_chi_square_w.iter())
        .chain(mean_and_ic_nq.iter())
        .chain(variance_and_ic_t_student_chi_square_nq.iter())
    {
        output_string += &format!("{},", num);
    }
    output_string += &format!("{:?}\n", policy);

    file.write_all(output_string.as_bytes())
        .expect("Failed to write csv file");
}
