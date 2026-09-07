use rand::Rng;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::task::JoinHandle;
use tokio::time::sleep;

#[derive(Clone, Default)]
pub struct PairKey {
    inner: Arc<Mutex<PairKeyInner>>,
}

#[derive(Default)]
struct PairKeyInner {
    key: Option<String>,
    handle: Option<JoinHandle<()>>,
    started_at: Option<u64>, // Timestamp UNIX en segundos
}

impl PairKey {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(PairKeyInner::default())),
        }
    }

    pub fn generate_and_start_timer<F>(&self, on_expire: F) -> String
    where
        F: Fn() + Send + 'static,
    {
        let mut guard = self.inner.lock().unwrap();

        if let Some(handle) = guard.handle.take() {
            handle.abort();
        }

        let mut rng = rand::thread_rng();
        let chars: [char; 36] = [
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G',
            'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X',
            'Y', 'Z',
        ];

        let key: String = (0..6)
            .map(|_| chars[rng.gen_range(0..chars.len())])
            .collect();

        // Obtener el tiempo Unix actual en segundos
        let now_unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("El tiempo retrocedió")
            .as_secs();

        guard.key = Some(key.clone());
        guard.started_at = Some(now_unix);

        let inner_clone = Arc::clone(&self.inner);

        let handle = tokio::spawn(async move {
            sleep(Duration::from_secs(10 * 60)).await;

            if let Ok(mut inner) = inner_clone.lock() {
                inner.key = None;
                inner.handle = None;
                inner.started_at = None;
            }

            on_expire();
        });

        guard.handle = Some(handle);
        key
    }

    /// Obtiene el timestamp UNIX (en segundos) de cuando inició la clave
    pub fn get_time(&self) -> Option<u64> {
        let guard = self.inner.lock().unwrap();
        guard.started_at
    }

    pub fn clear(&self) {
        let mut guard = self.inner.lock().unwrap();
        if let Some(handle) = guard.handle.take() {
            handle.abort();
        }
        guard.key = None;
        guard.started_at = None;
    }

    pub fn get_key(&self) -> Option<String> {
        let guard = self.inner.lock().unwrap();
        guard.key.clone()
    }
}