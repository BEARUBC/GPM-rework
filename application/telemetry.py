"""Telemetry collection and Prometheus export for GPM"""
from prometheus_client import Counter, Gauge, Histogram, start_http_server
from loguru import logger


class GpmTelemetry:
    """Collects and exports GPM metrics via Prometheus

    Metrics are exposed on an HTTP endpoint for scraping by Prometheus or
    viewing directly in a browser at http://<host>:<port>/metrics
    """

    def __init__(self, port=8080, prefix="gpm"):
        """Initialize telemetry system

        Args:
            port: HTTP port for metrics endpoint (default: 8080)
            prefix: Metric name prefix (default: "gpm")
        """
        self.port = port
        self.prefix = prefix

        # Hardware metrics - BMS
        self.battery_voltage = Gauge(
            f'{prefix}_battery_voltage_volts',
            'Battery voltage in volts'
        )
        self.battery_current = Gauge(
            f'{prefix}_battery_current_amps',
            'Battery current in amps'
        )
        self.battery_temperature = Gauge(
            f'{prefix}_battery_temperature_celsius',
            'Battery temperature in Celsius'
        )
        self.battery_charge = Gauge(
            f'{prefix}_battery_charge_percent',
            'Battery charge percentage'
        )

        # Operation metrics
        self.grip_changes = Counter(
            f'{prefix}_grip_changes_total',
            'Total grip changes',
            ['from_grip', 'to_grip']
        )
        self.safety_violations = Counter(
            f'{prefix}_safety_violations_total',
            'Safety constraint violations',
            ['type']
        )
        self.errors = Counter(
            f'{prefix}_errors_total',
            'Errors by component and type',
            ['component', 'error_type']
        )

        # Performance metrics
        self.loop_duration = Histogram(
            f'{prefix}_loop_duration_seconds',
            'Control loop iteration duration in seconds'
        )
        self.emg_read_duration = Histogram(
            f'{prefix}_emg_read_duration_seconds',
            'EMG read operation duration in seconds'
        )

        # State metrics
        self.current_state = Gauge(
            f'{prefix}_current_state',
            'Current state machine state (encoded)',
            ['state']
        )

    def start_server(self):
        """Start Prometheus metrics HTTP server

        The server runs in a background thread and serves metrics
        at http://0.0.0.0:<port>/metrics
        """
        try:
            start_http_server(self.port)
            logger.info(f"Telemetry server started on port {self.port}")
            logger.info(f"Metrics available at http://localhost:{self.port}/metrics")
        except OSError as e:
            if "address already in use" in str(e).lower():
                logger.warning(f"Telemetry port {self.port} already in use - metrics server not started")
            else:
                logger.error(f"Failed to start telemetry server: {e}")
        except Exception as e:
            logger.error(f"Failed to start telemetry server: {e}")

    def update_bms_metrics(self, bms_status):
        """Update battery metrics from BMS status

        Args:
            bms_status: BmsStatus object from gpm.Bms.get_status()
        """
        self.battery_voltage.set(bms_status.voltage)
        self.battery_current.set(bms_status.current)
        self.battery_temperature.set(bms_status.temperature)
        self.battery_charge.set(bms_status.charge_percentage)

    def record_grip_change(self, from_grip, to_grip):
        """Record a grip transition

        Args:
            from_grip: Previous grip type (str)
            to_grip: New grip type (str)
        """
        self.grip_changes.labels(from_grip=from_grip, to_grip=to_grip).inc()

    def record_safety_violation(self, violation_type):
        """Record a safety constraint violation

        Args:
            violation_type: Type of violation (e.g., "low_voltage", "high_temperature")
        """
        self.safety_violations.labels(type=violation_type).inc()

    def record_error(self, component, error_type):
        """Record an error occurrence

        Args:
            component: Component where error occurred (e.g., "main", "grip_controller")
            error_type: Type of error (e.g., "HardwareError", "SafetyError")
        """
        self.errors.labels(component=component, error_type=error_type).inc()

    def update_state(self, state_name):
        """Update current state machine state

        Args:
            state_name: Name of the current state (str)
        """
        # Clear previous state
        for state in ["IDLE", "ACTIVE", "CALIBRATING", "ERROR", "SHUTDOWN"]:
            self.current_state.labels(state=state).set(0)

        # Set current state
        self.current_state.labels(state=state_name).set(1)
