// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

using System.ComponentModel;

using Azure;
using Azure.DigitalTwins.Core;
using Microsoft.Extensions.Logging;

namespace Microsoft.ESDV.CloudConnector.Azure
{
    /// <summary>
    /// This class wraps the DigitalTwinsClient class in the Azure Digital Twins SDK
    /// Before calling the UpdateDigitalTwinAsync(...) method, you will need to be authenticated via your terminal by typing
    /// `az login --use-device-code --scope https://digitaltwins.azure.net/.default`
    /// </summary>
    public class DigitalTwinsClientWrapper
    {
        // The Azure Digital Twins Client.
        private readonly DigitalTwinsClient _client;

        // The logger.
        private readonly ILogger<DigitalTwinsClientWrapper> _logger;

        /// <summary>
        /// Checks if a path starts with a slash.
        /// </summary>
        /// <param name="path">the path.</param>
        /// <returns>Returns true if the path starts with a slash, otherwise false.</returns>
        private bool DoesPathStartsWithSlash(string path)
        {
            return path.StartsWith('/');
        }

        /// <summary>
        /// Constructor for DigitalTwinsClientWrapper
        /// </summary>
        /// <param name="client">A DigitalTwinsClient</param>
        /// <param name="logger">An ILogger</param>
        public DigitalTwinsClientWrapper(DigitalTwinsClient client, ILogger<DigitalTwinsClientWrapper> logger)
        {
            _client = client;
            _logger = logger;
            _logger.LogInformation("Starting Azure Digital Client");
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
        public async Task UpdateDigitalTwinAsync(string modelID, string instanceID, string instancePropertyPath, string data)
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

                    // First UpdateDigitalTwinAsync call may block due to initial authorization.
                    await _client.UpdateDigitalTwinAsync(instanceID, jsonPatchDocument);
                    _logger.LogInformation($"Successfully set instance {instanceID}{instancePropertyPath} based on model {modelID} to {data}");
                    return;
                }
                catch (RequestFailedException ex)
                {
                    _logger.LogError($"Cannot set instance {instanceID}{instancePropertyPath} based on model {modelID} to {data} due to {ex.Message}");
                    throw ex;
                }
                // Try to parse string data with the next type if we're unsuccessful.
                catch (Exception ex) when (ex is NotSupportedException || ex is ArgumentException || ex is FormatException)
                {
                    continue;
                }
            }

            string errorMessage = $"Failed to parse {data}. Cannot set instance {instanceID}{instancePropertyPath} based on model {modelID} to {data}";
            _logger.LogError(errorMessage);
            throw new NotSupportedException(errorMessage);
        }
    }
}
