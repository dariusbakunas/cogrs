use mac_address::get_mac_address;
use once_cell::sync::Lazy;
use rand::rngs::mock::StepRng;
use rand::{rng, Rng};
use std::fmt::Write;
use std::sync::{Arc, Mutex};

pub struct UniqueIdGenerator {
    cur_id: Mutex<u64>, // Internal counter for unique IDs
    node_mac: String,   // Cached MAC address for this node
}

impl UniqueIdGenerator {
    /// Creates a new `UniqueIdGenerator` and initializes the MAC address and counter.
    fn new() -> Self {
        let node_mac = Self::get_node_mac().unwrap_or_else(|| "001122334455".to_string());
        Self {
            cur_id: Mutex::new(0),
            node_mac,
        }
    }

    /// Generates a unique ID using the internal counter, MAC address, and a random number.
    pub fn generate_id<R: Rng>(&self, rng: &mut R) -> String {
        let random_int = rng.random_range(0..u32::MAX);

        let mut cur_id_lock = self.cur_id.lock().unwrap();
        *cur_id_lock += 1;

        let mut unique_id = String::new();
        write!(
            &mut unique_id,
            "{}-{}-{:04x}-{:04x}-{:012x}",
            &self.node_mac[0..8],
            &self.node_mac[8..12],
            random_int >> 16,    // Higher 16 bits
            random_int & 0xFFFF, // Lower 16 bits
            *cur_id_lock         // Incremental counter
        )
        .unwrap();

        unique_id
    }

    /// Internal helper: Fetches the MAC address dynamically.
    /// If no MAC address is found, returns `None`.
    fn get_node_mac() -> Option<String> {
        if let Ok(mac) = get_mac_address() {
            if let Some(mac) = mac {
                return Some(mac.bytes().iter().map(|b| format!("{:02x}", b)).collect());
            }
        }
        None
    }
}

/// Singleton instance of `UniqueIdGenerator`
static UNIQUE_ID_GENERATOR: Lazy<Arc<UniqueIdGenerator>> =
    Lazy::new(|| Arc::new(UniqueIdGenerator::new()));

/// Provides access to the Singleton instance of `UniqueIdGenerator`.
pub fn get_unique_id_generator() -> Arc<UniqueIdGenerator> {
    Arc::clone(&UNIQUE_ID_GENERATOR)
}

pub fn get_unique_id(predictable: bool) -> String {
    let generator = get_unique_id_generator();

    if predictable {
        let mut rng = StepRng::new(1, 0);
        generator.generate_id(&mut rng)
    } else {
        let mut rng = rng();
        generator.generate_id(&mut rng)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::mock::StepRng; // Mock RNG for predictable results
    use std::collections::HashSet;
    use std::sync::Arc;

    #[test]
    fn test_generate_id_with_mock_rng() {
        // Set up the generator
        let generator = UniqueIdGenerator::new();
        let mut mock_rng = StepRng::new(1, 0); // Mock RNG always returns 1

        // Generate ID
        let unique_id = generator.generate_id(&mut mock_rng);

        // Verify the unique ID follows the expected pattern and includes the mock RNG value
        assert!(
            unique_id.contains("0001"),
            "Generated ID does not include RNG value"
        );

        assert_eq!(
            &unique_id[0..8],
            &generator.node_mac[0..8],
            "First MAC segment mismatch in ID"
        );
        assert_eq!(
            &unique_id[9..13],
            &generator.node_mac[8..12],
            "Second MAC segment mismatch in ID"
        );
    }

    #[test]
    fn test_generate_id_increases_counter() {
        let generator = UniqueIdGenerator::new();
        let mut mock_rng = StepRng::new(1, 0);

        // Generate two IDs
        let id1 = generator.generate_id(&mut mock_rng);
        let id2 = generator.generate_id(&mut mock_rng);

        // Validate that the IDs are different due to incrementing counter
        assert_ne!(id1, id2, "IDs should differ as the counter increments");
        let counter1 = u64::from_str_radix(&id1[id1.len() - 12..], 16).unwrap();
        let counter2 = u64::from_str_radix(&id2[id2.len() - 12..], 16).unwrap();
        assert_eq!(
            counter2,
            counter1 + 1,
            "Counter did not increment correctly"
        );
    }

    #[test]
    fn test_generate_id_format() {
        let generator = UniqueIdGenerator::new();
        let mut mock_rng = StepRng::new(1, 0);

        // Generate an ID
        let unique_id = generator.generate_id(&mut mock_rng);

        // Validate the format of the generated ID: MAC-MAC-random-random-counter
        let parts: Vec<&str> = unique_id.split('-').collect();
        assert_eq!(parts.len(), 5, "Generated ID format is incorrect");
        assert_eq!(
            parts[0],
            &generator.node_mac[0..8],
            "First MAC segment is incorrect"
        );
        assert_eq!(
            parts[1],
            &generator.node_mac[8..12],
            "Second MAC segment is incorrect"
        );
        assert!(
            u32::from_str_radix(parts[2], 16).is_ok(),
            "Random number (part 3) is not valid hex"
        );
        assert!(
            u32::from_str_radix(parts[3], 16).is_ok(),
            "Random number (part 4) is not valid hex"
        );
        assert!(
            u64::from_str_radix(parts[4], 16).is_ok(),
            "Counter (part 5) is not valid hex"
        );
    }

    #[test]
    fn test_generate_id_thread_safety() {
        let mut handles = Vec::new();
        let generated_ids = Arc::new(Mutex::new(HashSet::new()));

        // Spawn multiple threads to generate unique IDs
        for _ in 0..10 {
            let generated_ids_clone = Arc::clone(&generated_ids);

            let handle = std::thread::spawn(move || {
                let unique_id = get_unique_id(true);

                // Insert the generated ID into the shared HashSet
                let mut ids = generated_ids_clone.lock().unwrap();
                if !ids.insert(unique_id) {
                    panic!("Duplicate ID generated in multi-threaded environment");
                }
            });

            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Assert uniqueness of all generated IDs
        let ids = generated_ids.lock().unwrap();
        assert_eq!(ids.len(), 10, "Not all generated IDs are unique");
    }
}
