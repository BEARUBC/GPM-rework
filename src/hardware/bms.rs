use super::Resource;
use crate::hal::traits::BmsDriver;

pub struct Bms {
    pub voltage: f32,
    pub current: f32,
    pub temperature: f32,
    pub is_healthy: bool,
}

#[derive(Clone, Debug)]
pub struct BmsStatus {
    pub voltage: f32,
    pub current: f32,
    pub temperature: f32,
    pub is_healthy: bool,
    pub charge_percentage: f32,
}

impl Resource for Bms {
    fn init() -> Self {
        Bms {
            voltage: 12.0,
            current: 0.0,
            temperature: 25.0,
            is_healthy: true,
        }
    }

    fn name() -> String {
        "Bms".to_string()
    }
}

impl Bms {
    pub fn get_status(&self) -> BmsStatus {
        BmsStatus {
            voltage: self.voltage,
            current: self.current,
            temperature: self.temperature,
            is_healthy: self.is_healthy,
            charge_percentage: self.calculate_charge_percentage(),
        }
    }

    pub fn update(&mut self) {
        // TODO: Implement actual BMS reading
        // For now, simulate healthy battery
        #[cfg(not(feature = "pi"))]
        {
            use rand::Rng;
            let mut rng = rand::rng();
            self.voltage = 11.5 + rng.random::<f32>() * 1.0; // 11.5-12.5V
            self.current = rng.random::<f32>() * 2.0; // 0-2A
            self.temperature = 20.0 + rng.random::<f32>() * 10.0; // 20-30°C
            self.is_healthy = self.voltage > 10.0 && self.temperature < 50.0;
        }
    }

    fn calculate_charge_percentage(&self) -> f32 {
        // Simple linear approximation: 10V = 0%, 12.6V = 100%
        let min_voltage = 10.0;
        let max_voltage = 12.6;

        ((self.voltage - min_voltage) / (max_voltage - min_voltage) * 100.0)
            .max(0.0)
            .min(100.0)
    }
}

impl BmsDriver for Bms {
    fn get_status(&self) -> BmsStatus {
        Bms::get_status(self)
    }

    fn update(&mut self) {
        Bms::update(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_charge_percentage_calculation() {
        let mut bms = Bms::init();
        bms.voltage = 10.0;
        assert_eq!(bms.calculate_charge_percentage(), 0.0);

        bms.voltage = 12.6;
        assert_eq!(bms.calculate_charge_percentage(), 100.0);

        bms.voltage = 11.3;
        assert_eq!(bms.calculate_charge_percentage(), 50.0);
    }

    #[test]
    fn test_charge_calculation_out_of_range() {
        let mut bms = Bms::init();
        bms.voltage = 9.9;
        assert_eq!(bms.calculate_charge_percentage(), 0.0);

        bms.voltage = 12.7;
        assert_eq!(bms.calculate_charge_percentage(), 100.0);
    }

    #[test]
    fn test_health_status_logic() {
        let mut bms = Bms::init();
        bms.voltage = 11.0;
        bms.temperature = 20.0;
        assert!(bms.is_healthy);

        bms.voltage = 10.0;
        assert!(!bms.is_healthy);

        bms.voltage = 10.0;
        bms.temperature = 50.0;
        assert!(!bms.is_healthy);
        // TODO: this one fails b/c update isn't properly implemented
    }

    #[test]
    fn test_status_struct_population() {
        let mut bms = Bms::init();
        bms.voltage = 10.0;
        bms.temperature = 20.0;
        bms.is_healthy = true;

        let status = bms.get_status();
        assert_eq!(status.voltage, 10.0);
        assert_eq!(status.temperature, 20.0);
        assert!(status.is_healthy);
    }
}
