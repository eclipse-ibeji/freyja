// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Threading.Tasks;

using Azure;
using Azure.DigitalTwins.Core;
using Azure.Identity;
using Azure.Messaging;
using Microsoft.Azure.WebJobs;
using Microsoft.Azure.WebJobs.Extensions.EventGrid;
using Microsoft.Extensions.Logging;

namespace Microsoft.ESDV.CloudConnector.Azure {
    /// <summary>
    /// This class contains the info to target an Azure Digital Twin instance.
    /// </summary>
    public class DigitalTwinsInstance {
        public string model_id { get; set; }
        public string instance_id { get; set; }
        public string instance_property_path { get; set; }
        public string data { get; set; }
    }

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
        /// <param name="client">the Azure Digital Twins client.</param>
        /// <param name="instance">the digital twin instance to update.</param>
        /// <returns>Returns a task for updating a digital twin instance.</returns>
        public static async Task UpdateDigitalTwinAsync(DigitalTwinsClient client, DigitalTwinsInstance instance)
        {
            List<Type> dataTypes = new List<Type>() { typeof(Double), typeof(Boolean), typeof(Int32) };
            var jsonPatchDocument = new JsonPatchDocument();
            string errorMessage = null;

            foreach (Type type in dataTypes)
            {
                try
                {
                    // Parse the data to a type
                    dynamic value = TypeDescriptor.GetConverter(type).ConvertFromInvariantString(instance.data);

                    if (!DoesPathStartsWithSlash(instance.instance_property_path))
                    {
                        instance.instance_property_path = $"/{instance.instance_property_path}";
                    }
                    // Once we're able to parse the data to a type
                    // we append it to the jsonPatchDocument
                    jsonPatchDocument.AppendAdd(instance.instance_property_path, value);
                }
                // Try to parse the data using the next type
                catch (Exception ex) when (ex is NotSupportedException || ex is ArgumentException || ex is FormatException)
                {
                    continue;
                }

                try
                {
                    // Exit the function if the instance is successfully updated.
                    await client.UpdateDigitalTwinAsync(instance.instance_id, jsonPatchDocument);
                    return;
                }
                catch(RequestFailedException ex)
                {
                    errorMessage = @$"Failed to parse {instance.data} due to {ex.Message}.
                        Cannot set instance {instance.instance_id}{instance.instance_property_path}
                        based on model {instance.model_id} to {instance.data}";
                }
            }

            throw new NotSupportedException(errorMessage);
        }

        /// <summary>
        /// An Azure Function that updates an Azure Digital Twin based on the request.
        /// </summary>
        /// <param name="cloudEvent">the cloudEvent request that is received.</param>
        /// <param name="logger">the logger</param>
        /// <exception>An exception is thrown if the Azure Digital Twin client cannot update an instance.</exception>
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
                await UpdateDigitalTwinAsync(client, instance);
                logger.LogInformation(@$"Successfully set instance {instance.instance_id}{instance.instance_property_path}
                    based on model {instance.model_id} to {instance.data}");
            }
            catch (Exception ex)
            {
                logger.LogError(ex.Message);
                throw;
            }
        }
    }
}
