# Build stage using .NET 9 SDK (ARM64)
FROM mcr.microsoft.com/dotnet/sdk:9.0 AS build
WORKDIR /app

# Copy project files and restore dependencies
COPY *.csproj ./
RUN dotnet restore

# Copy the rest of the source code
COPY . ./

# Publish the app for release
RUN dotnet publish -c Release -o out

# Runtime stage using .NET 9 ASP.NET runtime (ARM64)
FROM mcr.microsoft.com/dotnet/aspnet:9.0
WORKDIR /app

# Copy published app from build stage
COPY --from=build /app/out ./

# Expose port (change if needed)
EXPOSE 80

# Run the app
ENTRYPOINT ["dotnet", "weatherman.dll"]