namespace weatherman;

using System.Text.Json.Serialization;
using InfluxData.Net.InfluxDb;

public record WeatherData(double Temperature, double Humidity, IReadOnlyList<double>? TemperatureHistory = null)
{
    [JsonPropertyName("temperature")]
    public double Temperature { get; init; } = Temperature;
    [JsonPropertyName("humidity")]
    public double Humidity { get; init; } = Humidity;
    [JsonPropertyName("history")]
    public IReadOnlyList<double> TemperatureHistory { get; init; } = TemperatureHistory ?? [];
    [JsonPropertyName("dew_point")]
    public double DewPoint => Humidity > 0 ? Temperature - ((100 - Humidity) / 5) : 0;
}

public class WeatherDataService
{
    private readonly InfluxDbClient influxDbClient;
    private readonly string database;

    public WeatherDataService(InfluxDbClient influxDbClient, IConfiguration configuration)
    {
        this.influxDbClient = influxDbClient;
        this.database = configuration["INFLUXDB_DATABASE"] ?? "weather";
    }

    public async Task<WeatherData?> GetLatestWeatherAsync()
    {
        var result = await influxDbClient.Client.QueryAsync("SELECT last(temperature) AS temperature, last(humidity) AS humidity FROM weather", database);
        if (result.SingleOrDefault() is { Values: [[_, double temperature, long humidity]] })
        {
            if ((temperature as double?, humidity as long?) is (double t, long h))
            {
                return new WeatherData(t, h);
            }
        }
        return null;
    }
    
    public async Task<IReadOnlyList<double>> GetTemperatureHistoryAsync()
    {
        var result = await influxDbClient.Client.QueryAsync("""
            SELECT mean(temperature) FROM weather 
            WHERE time >= now() - 24h 
            GROUP BY time(10m) fill(linear)
            """, database);

        return result.SingleOrDefault()?.Values.Select(v => v[1] as double?).FillNullsWithPrevious().ToList() ?? [];
    }

    public async Task<WeatherData?> GetWeatherDataAsync()
    {
        var latestWeather = await GetLatestWeatherAsync();
        if (latestWeather is null)
        {
            return null;
        }
        var temperatureHistory = await GetTemperatureHistoryAsync();
        return new WeatherData(latestWeather.Temperature, latestWeather.Humidity, temperatureHistory);
    }
}