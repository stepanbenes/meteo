using InfluxData.Net.Common.Enums;
using InfluxData.Net.InfluxDb;
using weatherman;

var builder = WebApplication.CreateBuilder(args);

var influxDbUrl = builder.Configuration["INFLUXDB_URL"];
var influxDbDatabase = builder.Configuration["INFLUXDB_DATABASE"];

builder.Services.AddScoped(_ =>
    new InfluxDbClient(influxDbUrl, "", "", InfluxDbVersion.v_1_3) // no token required in 1.x
);

builder.Services.AddHostedService<PeriodicDashboardUpdateService>();
builder.Services.AddHttpClient();
builder.Services.AddScoped<WeatherDataService>();

var app = builder.Build();

app.UseDeveloperExceptionPage();
app.UseHttpsRedirection();
app.UseDefaultFiles();
app.UseStaticFiles();

// TODO: use WeatherDataService to get the latest weather data
app.MapGet("/api/weather", async (InfluxDbClient influx) =>
{
    //var result = await influx.Client.QueryAsync("SELECT MEAN(temperature) FROM weather WHERE time > now() - 1h", influxDbDatabase);
    var result = await influx.Client.QueryAsync("SELECT last(temperature) AS temperature, last(humidity) AS humidity FROM weather", influxDbDatabase);
    if (result.SingleOrDefault() is { Values:[[_, double temperature, long humidity]] })
    {
        var type_t = temperature as double?;
        var type_h = humidity as long?;
        if ((temperature as double?, humidity as long?) is (double t, long h))
        {
            return Results.Ok(new WeatherData(t, h));
        }
    }
    return Results.NotFound("No data");
});

// TODO: use WeatherDataService to get the latest weather data
app.MapGet("/api/temperature", async (InfluxDbClient influx) =>
{
    var result = await influx.Client.QueryAsync("""
        SELECT mean(temperature) FROM weather 
        WHERE time >= now() - 24h 
        GROUP BY time(10m) fill(linear)
        """, influxDbDatabase);
    return Results.Ok(result);
});

app.Run();
