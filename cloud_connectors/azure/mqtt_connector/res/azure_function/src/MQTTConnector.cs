// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

using System;
using System.Collections.Generic;
using System.Threading.Tasks;

using Microsoft.Azure.WebJobs;
using Microsoft.Extensions.Logging;
using Microsoft.Azure.WebJobs.Extensions.EventGrid;
using Azure;
using Azure.Messaging;
using Azure.Identity;
using Azure.DigitalTwins.Core;
using System.ComponentModel;

/// <summary>
/// This class contains the info to target an Azure Digital Twin instance.
/// </summary>
class DigitalTwinsInstance {
    public string model_id { get; set; }
    public string instance_id { get; set; }
    public string instance_property_path { get; set; }
    public string data { get; set; }
}

namespace MQTTConnector {
    public static class MQTTConnectorAzureFn {
        /// <summary>
        /// An Azure Function that updates an Azure Digital Twin based on the request.
        /// </summary>
        /// <param name="cloudEvent">the cloudEvent request that is received.</param>
        /// <param name="logger">the logger</param>
        /// <exception>An exception is thrown if the digital twin client cannot perform an update.</exception>
        /// <returns></returns>
        [FunctionName("MQTTConnectorAzureFn")]
        public static async Task Run([EventGridTrigger] CloudEvent cloudEvent, ILogger logger)
        {
            List<Type> dataTypes = new List<Type>() { typeof(Double), typeof(Boolean), typeof(Int32) };

            foreach (Type type in dataTypes)
            {
                var jsonPatchDocument = new JsonPatchDocument();
                DigitalTwinsInstance instance = cloudEvent.Data.ToObjectFromJson<DigitalTwinsInstance>();
                try
                {
                    dynamic value = TypeDescriptor.GetConverter(type).ConvertFromInvariantString(instance.data);
                    jsonPatchDocument.AppendAdd(instance.instance_property_path, value);
                }
                // Try to parse string data with the next type if we're unsuccessful.
                catch (Exception ex) when (ex is NotSupportedException || ex is ArgumentException || ex is FormatException)
                {
                    continue;
                }

                try
                {
                    var credential = new DefaultAzureCredential();
                    var adt_instance_url = Environment.GetEnvironmentVariable("KEYVAULT_SETTINGS", EnvironmentVariableTarget.Process);
                    var client = new DigitalTwinsClient(new Uri(adt_instance_url), credential);
                    await client.UpdateDigitalTwinAsync(instance.instance_id, jsonPatchDocument);
                }
                catch(Exception ex)
                {
                    logger.LogError($"Cannot set instance due to {ex.Message}");
                    break;
                }

                logger.LogInformation("Successfully set instance: {instance_id}{instance_property_path} based on model {model} to {data}", instance.instance_id, instance.instance_property_path, instance.model_id, instance.data);
                return;
            }

            string errorMessage = $"Failed to parse {cloudEvent.Data.ToString()}";
            logger.LogError(errorMessage);
            throw new NotSupportedException(errorMessage);
        }
    }
}
