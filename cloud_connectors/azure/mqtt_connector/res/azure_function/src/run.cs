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

namespace Microsoft.ESDV.CloudConnector.Azure {
    public static class MQTTConnectorAzureFunction {
        /// <summary>
        /// Checks if a path starts with a slash.
        /// </summary>
        /// <param name="path">the path.</param>
        /// <returns>Returns true if the path starts with a slash, otherwise false.</returns>
        public static bool DoesPathStartsWithSlash(string path)
        {
            return path.StartsWith('/');
        }

        /// <summary>
        /// Updates a digital twin's property.
        /// </summary>
        /// <example>
        /// Invoking <code>UpdateDigitalTwinAsync("dtmi:sdv:Cloud:Vehicle:Cabin:HVAC:AmbientAirTemperature;1", "44")</code>
        /// sets the dtmi "dtmi:sdv:Cloud:Vehicle:Cabin:HVAC:AmbientAirTemperature;1" to 44.
        /// </example>
        /// <param name="modelID">the model ID that a digital twin instance is based on.</param>
        /// <param name="instanceID">the digital twin instance ID.</param>
        /// <param name="instancePropertyPath">the property path of a digital twin instance to update.</param>
        /// <param name="data">the data used to update a digital twin instance's property.</param>
        /// <returns>Returns a task for updating a digital twin instance.</returns>
        public static async Task UpdateDigitalTwinAsync(DigitalTwinsClient client, ILogger logger, string modelID, string instanceID, string instancePropertyPath, string data)
        {
            List<Type> dataTypes = new List<Type>() { typeof(Double), typeof(Boolean), typeof(Int32) };
            var jsonPatchDocument = new JsonPatchDocument();

            foreach (Type type in dataTypes)
            {
                try
                {
                    // Parse the data string to a type
                    dynamic value = TypeDescriptor.GetConverter(type).ConvertFromInvariantString(data);

                    if (!DoesPathStartsWithSlash(instancePropertyPath))
                    {
                        instancePropertyPath = "$/{instancePropertyPath}";
                    }
                    // Once we're able to parse the data string to a type
                    // we append it to the jsonPatchDocument
                    jsonPatchDocument.AppendAdd(instancePropertyPath, value);
                }
                // Try to parse string data with the next type if we're unsuccessful.
                catch (Exception ex) when (ex is NotSupportedException || ex is ArgumentException || ex is FormatException)
                {
                    continue;
                }
            }

            try
            {
                await client.UpdateDigitalTwinAsync(instanceID, jsonPatchDocument);
            }
            catch(Exception ex)
            {
                string errorMessage = @$"Failed to parse {data} due to {ex.Message}.
                    Cannot set instance {instanceID}{instancePropertyPath} based on model {modelID} to {data}";
                throw new NotSupportedException(errorMessage);
            }
            logger.LogInformation($"Successfully set instance {instanceID}{instancePropertyPath} based on model {modelID} to {data}");
        }

        /// <summary>
        /// An Azure Function that updates an Azure Digital Twin based on the request.
        /// </summary>
        /// <param name="cloudEvent">the cloudEvent request that is received.</param>
        /// <param name="logger">the logger</param>
        /// <exception>An exception is thrown if the digital twin client cannot perform an update.</exception>
        /// <returns></returns>
        [FunctionName("MQTTConnectorAzureFunction")]
        public static async Task Run([EventGridTrigger] CloudEvent cloudEvent, ILogger logger)
        {
            List<Type> dataTypes = new List<Type>() { typeof(Double), typeof(Boolean), typeof(Int32) };
            DigitalTwinsInstance instance = cloudEvent.Data.ToObjectFromJson<DigitalTwinsInstance>();

            try
            {
                var credential = new DefaultAzureCredential();
                var adt_instance_url = Environment.GetEnvironmentVariable("KEYVAULT_SETTINGS", EnvironmentVariableTarget.Process);
                var client = new DigitalTwinsClient(new Uri(adt_instance_url), credential);
                await UpdateDigitalTwinAsync(client, logger, instance.model_id, instance.instance_id, instance.instance_property_path, instance.data);
            }
            catch (Exception ex)
            {
                logger.LogError(ex.Message);
                throw;
            }
        }
    }
}
