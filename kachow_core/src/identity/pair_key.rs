use rand::Rng;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio::time::sleep;

#[derive(Clone, Default)]
pub struct PairKey {
    // Estado compartido protegido por Mutex para permitir mutación desde tokio::spawn
    inner: Arc<Mutex<PairKeyInner>>,
}

#[derive(Default)]
struct PairKeyInner {
    key: Option<String>,
    handle: Option<JoinHandle<()>>,
}

impl PairKey {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(PairKeyInner::default())),
        }
    }

    /// Genera la clave, la almacena y programa la expiración a los 10 minutos.
    /// Recibe un closure que se ejecuta cuando se cumple el tiempo (para poner mode = false).
    pub fn generate_and_start_timer<F>(&self, on_expire: F) -> String
    where
        F: Fn() + Send + 'static,
    {
        let mut guard = self.inner.lock().unwrap();

        // Si había un temporizador corriendo, lo abortamos
        if let Some(handle) = guard.handle.take() {
            handle.abort();
        }

        // Generar clave de 6 caracteres
        let mut rng = rand::thread_rng();
        let chars: [char; 36] = [
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G',
            'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X',
            'Y', 'Z',
        ];

        let key: String = (0..6)
            .map(|_| chars[rng.gen_range(0..chars.len())])
            .collect();

        guard.key = Some(key.clone());

        // Clonamos el Arc interno para pasarlo a la tarea en segundo plano
        let inner_clone = Arc::clone(&self.inner);

        // Lanza la tarea asíncrona de 10 minutos
        let handle = tokio::spawn(async move {
            sleep(Duration::from_secs(10 * 60)).await;

            // Al expirar: limpiamos la clave
            if let Ok(mut inner) = inner_clone.lock() {
                inner.key = None;
                inner.handle = None;
            }

            // Ejecutamos el callback para cambiar el estado global (mode = false)
            on_expire();
        });

        guard.handle = Some(handle);
        key
    }

    /// Limpia la clave y cancela el temporizador activo
    pub fn clear(&self) {
        let mut guard = self.inner.lock().unwrap();
        if let Some(handle) = guard.handle.take() {
            handle.abort();
        }
        guard.key = None;
    }

    /// Obtiene la clave actual si está activa
    pub fn get_key(&self) -> Option<String> {
        let guard = self.inner.lock().unwrap();
        guard.key.clone()
    }
}
