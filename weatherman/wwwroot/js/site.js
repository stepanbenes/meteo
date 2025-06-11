function updateTimestamp() {
    const now = new Date();
    const timestamp = now.toLocaleString('en-US', {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
        hour12: false
    });
    document.getElementById('timestamp').textContent = `SYSTEM TIME: ${timestamp}`;
}

function simulateDataUpdate() {
    // Simulate changing weather data
    const temp = (20 + Math.random() * 10).toFixed(1);
    const humidity = Math.floor(50 + Math.random() * 40);
    const feelsLike = (parseFloat(temp) + Math.random() * 4 - 2).toFixed(1);

    // Simulate sunrise/sunset times (varies by season)
    const sunriseHour = 6 + Math.floor(Math.random() * 2); // 6-7 AM
    const sunriseMin = Math.floor(Math.random() * 60);
    const sunsetHour = 19 + Math.floor(Math.random() * 2); // 7-8 PM  
    const sunsetMin = Math.floor(Math.random() * 60);

    const sunrise = `${sunriseHour.toString().padStart(2, '0')}:${sunriseMin.toString().padStart(2, '0')}`;
    const sunset = `${sunsetHour.toString().padStart(2, '0')}:${sunsetMin.toString().padStart(2, '0')}`;

    // Moon phase simulation
    const phases = [
        { name: "NEW MOON", ascii: "░░░░░░░\n░░░░░░░\n░░███░░\n░░███░░\n░░░░░░░\n░░░░░░░", illum: "0%" },
        { name: "WAXING CRESCENT", ascii: "░░░░░░░\n░░░▒▒░░\n░░▒▒▒░░\n░░▒▒▒░░\n░░░▒▒░░\n░░░░░░░", illum: "25%" },
        { name: "FIRST QUARTER", ascii: "░░░░░░░\n░░░▒▒▒░\n░░▒▒▒▒░\n░░▒▒▒▒░\n░░░▒▒▒░\n░░░░░░░", illum: "50%" },
        { name: "WAXING GIBBOUS", ascii: "░░▒▒▒░░\n░▒▒▒▒▒░\n▒▒▒▒▒▒▒\n▒▒▒▒▒▒▒\n░▒▒▒▒▒░\n░░▒▒▒░░", illum: "78%" },
        { name: "FULL MOON", ascii: "░▒▒▒▒▒░\n▒▒▒▒▒▒▒\n▒▒▒▒▒▒▒\n▒▒▒▒▒▒▒\n▒▒▒▒▒▒▒\n░▒▒▒▒▒░", illum: "100%" }
    ];

    const currentPhase = phases[Math.floor(Math.random() * phases.length)];

    document.getElementById('temperature').textContent = temp;
    document.getElementById('humidity').textContent = humidity;
    document.getElementById('feels-like').textContent = feelsLike + '°C';
    document.getElementById('sunrise').textContent = sunrise;
    document.getElementById('sunset').textContent = sunset;
    document.getElementById('moon-phase').textContent = currentPhase.name;
    document.getElementById('moon-illumination').textContent = currentPhase.illum;

    // Update humidity status
    const humidityStatus = humidity < 30 ? "DRY" : humidity > 70 ? "HUMID" : "COMFORTABLE";
    document.getElementById('humidity-status').textContent = humidityStatus;
}

function readWeatherData() {
    // fetch weather data from the server
    fetch('/api/weather')
        .then(response => response.json())
        .then(data => {
            document.getElementById('temperature').textContent = data.temperature;
            document.getElementById('humidity').textContent = data.humidity;

            // Update humidity status
            const humidityStatus = data.humidity < 30 ? "DRY" : data.humidity > 70 ? "HUMID" : "COMFORTABLE";
            document.getElementById('humidity-status').textContent = humidityStatus;

            // Update dew point
            document.getElementById('dew-point').textContent = data.dewPoint + '°C';
        })
        .catch(error => console.error('Error fetching weather data:', error));
}

window.onload = function() {
    // Update timestamp every second
    setInterval(updateTimestamp, 1000);
    updateTimestamp();

    // Fetch new data every minute
    setInterval(readWeatherData, 60_000);
    // Initial data fetch
    readWeatherData();
};