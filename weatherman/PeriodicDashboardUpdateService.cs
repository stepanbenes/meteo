namespace weatherman;
using InfluxData.Net.InfluxDb;

public class PeriodicDashboardUpdateService : BackgroundService
{
    private readonly ILogger<PeriodicDashboardUpdateService> logger;
    private readonly IServiceProvider serviceProvider;
    private readonly IHttpClientFactory httpClientFactory;
    private readonly WeatherDataService weatherDataService;
    private readonly TimeSpan interval = TimeSpan.FromMinutes(10); // adjust as needed

    public PeriodicDashboardUpdateService(
        ILogger<PeriodicDashboardUpdateService> logger,
        IServiceProvider serviceProvider,
        IHttpClientFactory httpClientFactory,
        WeatherDataService weatherDataService)
    {
        this.logger = logger;
        this.serviceProvider = serviceProvider;
        this.httpClientFactory = httpClientFactory;
        this.weatherDataService = weatherDataService;
    }

    protected override async Task ExecuteAsync(CancellationToken stoppingToken)
    {
        while (!stoppingToken.IsCancellationRequested)
        {
            try
            {
                using (var scope = serviceProvider.CreateScope())
                {
                    var influxDb = scope.ServiceProvider.GetRequiredService<InfluxDbClient>();

                    //var dataToSend = await weatherDataService.GetLatestWeatherAsync();
                    var dataToSend = await weatherDataService.GetWeatherDataAsync();

                    logger.LogInformation("Sending periodic data: {Data}", dataToSend);
                    logger.LogInformation("Temperature history: {History}", dataToSend?.TemperatureHistory);

                    if (dataToSend is not null)
                    {
                        if (dataToSend.TemperatureHistory is [_])
                        {
                            dataToSend = dataToSend with { TemperatureHistory = [] }; // avoid sending single history point
                        }
                        var httpClient = httpClientFactory.CreateClient();
                        var response = await httpClient.PostAsJsonAsync("http://morty:5000/update", dataToSend, stoppingToken);
                        response.EnsureSuccessStatusCode();
                    }
                }
            }
            catch (Exception ex)
            {
                logger.LogError(ex, "Error while sending periodic data.");
            }

            await Task.Delay(interval, stoppingToken);
        }
    }
}
