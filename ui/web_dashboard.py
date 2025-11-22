"""Web dashboard for GPM (optional)"""
try:
    from flask import Flask, render_template, jsonify
    FLASK_AVAILABLE = True
except ImportError:
    FLASK_AVAILABLE = False
    print("Flask not installed. Install with: pip install flask")

from application.hardware import HardwareInterface

if FLASK_AVAILABLE:
    app = Flask(__name__)
    hardware = None

    @app.route('/')
    def index():
        """Main dashboard"""
        return """
        <html>
        <head>
            <title>GPM Dashboard</title>
            <style>
                body { font-family: Arial, sans-serif; margin: 20px; }
                .status { padding: 10px; margin: 10px 0; border: 1px solid #ccc; }
                .healthy { background-color: #d4edda; }
                .warning { background-color: #fff3cd; }
                .error { background-color: #f8d7da; }
            </style>
        </head>
        <body>
            <h1>GPM Dashboard</h1>
            <div id="status"></div>
            <script>
                function updateStatus() {
                    fetch('/api/status')
                        .then(r => r.json())
                        .then(data => {
                            document.getElementById('status').innerHTML = 
                                '<div class="status ' + 
                                (data.bms.is_healthy ? 'healthy' : 'error') + '">' +
                                '<h2>BMS Status</h2>' +
                                '<p>Voltage: ' + data.bms.voltage.toFixed(2) + 'V</p>' +
                                '<p>Current: ' + data.bms.current.toFixed(2) + 'A</p>' +
                                '<p>Temperature: ' + data.bms.temperature.toFixed(1) + '°C</p>' +
                                '<p>Charge: ' + data.bms.charge_percentage.toFixed(1) + '%</p>' +
                                '</div>' +
                                '<div class="status">' +
                                '<h2>EMG Status</h2>' +
                                '<p>Ready: ' + (data.emg_ready ? 'Yes' : 'No') + '</p>' +
                                '</div>';
                        });
                }
                updateStatus();
                setInterval(updateStatus, 1000);
            </script>
        </body>
        </html>
        """

    @app.route('/api/status')
    def status():
        """API endpoint for status"""
        global hardware
        if hardware is None:
            hardware = HardwareInterface()
            hardware.initialize()
        
        return jsonify(hardware.get_status())

    def run_dashboard(host='0.0.0.0', port=5000):
        """Start the web dashboard"""
        print(f"Starting dashboard on http://{host}:{port}")
        app.run(host=host, port=port, debug=False)

else:
    def run_dashboard(host='0.0.0.0', port=5000):
        print("Flask not available. Cannot start dashboard.")


if __name__ == "__main__":
    run_dashboard()
