using InfluxData.Net.Common.Enums;
using InfluxData.Net.InfluxDb;

var builder = WebApplication.CreateBuilder(args);

var influxDbUrl = builder.Configuration["INFLUXDB_URL"];
var influxDbDatabase = builder.Configuration["INFLUXDB_DATABASE"];

builder.Services.AddSingleton(_ =>
    new InfluxDbClient(influxDbUrl, "", "", InfluxDbVersion.v_1_3) // no token required in 1.x
);

var app = builder.Build();

app.UseDeveloperExceptionPage();
app.UseHttpsRedirection();
app.UseDefaultFiles();
app.UseStaticFiles();

var summaries = new[]
{
    "Freezing", "Bracing", "Chilly", "Cool", "Mild", "Warm", "Balmy", "Hot", "Sweltering", "Scorching"
};

app.MapGet("/weatherforecast", () =>
{
    var forecast =  Enumerable.Range(1, 5).Select(index =>
        new WeatherForecast
        (
            DateOnly.FromDateTime(DateTime.Now.AddDays(index)),
            Random.Shared.Next(-20, 55),
            summaries[Random.Shared.Next(summaries.Length)]
        ))
        .ToArray();
    return forecast;
})
.WithName("GetWeatherForecast");

app.MapGet("/current-weather", async (InfluxDbClient influx) =>
{
    //var result = await influx.Client.QueryAsync("SELECT MEAN(temperature) FROM weather WHERE time > now() - 1h", influxDbDatabase);
    var result = await influx.Client.QueryAsync("SELECT last(temperature), last(humidity), time FROM weather", influxDbDatabase);
    return Results.Ok(result);
});

app.MapGet("/temperature", async (InfluxDbClient influx) =>
{
    var result = await influx.Client.QueryAsync("""
        SELECT mean(temperature) FROM weather 
        WHERE time >= now() - 24h 
        GROUP BY time(10m) fill(linear)
        """, influxDbDatabase);
    return Results.Ok(result);
});

app.Run();

record WeatherForecast(DateOnly Date, int TemperatureC, string? Summary)
{
    public int TemperatureF => 32 + (int)(TemperatureC / 0.5556);
}
