// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

using System.Text.Json;

using Azure.DigitalTwins.Core;
using Azure.Identity;

using Microsoft.ESDV.CloudConnector.Azure.GrpcService.Services;

namespace Microsoft.ESDV.CloudConnector.Azure
{
    class AzureDigitalTwinsInstanceConfig
    {
        public string AzureDigitalTwinsInstanceUrl { get; set; }
    }

    class Program
    {
        static void Main(string[] args)
        {
            const string AZURE_DIGITAL_TWINS_INSTANCE_CONFIG_PATH = @"bin/Debug/net6.0/config/adt_instance_config.json";
            string contents = File.ReadAllText(AZURE_DIGITAL_TWINS_INSTANCE_CONFIG_PATH);
            AzureDigitalTwinsInstanceConfig adtInstanceConfig = JsonSerializer.Deserialize<AzureDigitalTwinsInstanceConfig>(contents);

            // Configure the builder
            WebApplicationBuilder builder = WebApplication.CreateBuilder(args);

            string adtInstanceUrl = adtInstanceConfig.AzureDigitalTwinsInstanceUrl;
            var credential = new DefaultAzureCredential();
            DigitalTwinsClient client = new DigitalTwinsClient(new Uri(adtInstanceUrl), credential);

            ILoggerFactory loggerFactory = LoggerFactory.Create(builder => builder.AddSimpleConsole(c =>
            {
                c.TimestampFormat = "[yyyy-MM-ddTHH:mm::ssZ] ";
                c.UseUtcTimestamp = true;
            }));
            loggerFactory.CreateLogger("Main").LogInformation("Started the Azure Digital Twins Connector");

            // Instantiate the DigitalTwinClient first before adding it as a service for dependency injection.
            // Otherwise, if the constructor throws an exception due to invalid configurations, this exception
            // would be handled by the GRPC service every time a new request is sent from the client, so the program won't crash.
            // This is not ideal since we should fail fast with invalid configurations.
            builder.Host.ConfigureLogging(logging =>
            {
                logging.ClearProviders();
                logging.AddSimpleConsole(settings =>
                {
                    settings.TimestampFormat = "[yyyy-MM-ddTHH:mm:ssZ] ";
                    settings.UseUtcTimestamp = true;
                });
            });

            builder.Services.AddSingleton(client);
            builder.Services.AddSingleton<DigitalTwinsClientWrapper>();
            builder.Services.AddGrpc(options => options.EnableDetailedErrors = true);

            WebApplication app = builder.Build();
            app.MapGrpcService<DigitalTwinsConnectorService>();
            app.Run();
        }
    }
}
