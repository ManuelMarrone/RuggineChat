use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

pub fn start_log(duration: Arc<Mutex<Duration>>) {
    tokio::spawn(async move {
        loop {
            // Copia il valore sotto lock e rilascia subito il mutex
            let secs = {
                let d = duration.lock().unwrap();
                d.as_secs_f64()
            };

            let line = format!("Durata corrente: {:.6} s\n", secs);

            // Crea la directory se non esiste
            if let Err(e) = tokio::fs::create_dir_all("Log").await {
                eprintln!("Errore creazione directory: {}", e);
                continue;
            }

            // Apri il file in append e scrivi
            match OpenOptions::new()
                .write(true)
                .create(true)
                .append(true)
                .open("Log/cpu_log.txt")
                .await
            {
                Ok(mut file) => {
                    if let Err(e) = file.write_all(line.as_bytes()).await {
                        eprintln!("Errore scrittura file: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Errore apertura file: {}", e);
                }
            }

            tokio::time::sleep(Duration::from_secs(120)).await;
        }
    });
}
