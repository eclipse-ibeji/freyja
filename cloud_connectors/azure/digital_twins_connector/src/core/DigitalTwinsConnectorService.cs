// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

using Grpc.Core;

namespace Microsoft.ESDV.CloudConnector.Azure.GrpcService.Services
{
    /// <summary>
    /// This class implements the gRPC service for the Azure Cloud Connector
    /// </summary>
    public class DigitalTwinsConnectorService : AzureCloudConnector.AzureCloudConnectorBase
    {
        // The logger.
        private readonly ILogger<DigitalTwinsConnectorService> _logger;

        // Used to update the cloud digital twin instances' values.
        private readonly DigitalTwinsClientWrapper _digitalTwinClient;

        /// <summary>
        /// Constructor for DigitalTwinsConnectorService
        /// </summary>
        /// <param name="logger"></param>
        /// <param name="digitalTwinClient"></param>
        public DigitalTwinsConnectorService(ILogger<DigitalTwinsConnectorService> logger, DigitalTwinsClientWrapper digitalTwinClient)
        {
            _logger = logger;
            _digitalTwinClient = digitalTwinClient;
        }

        /// <summary>
        /// Updates an Azure Digital Twin instance.
        /// </summary>
        /// <param name="request">the request to send.</param>
        /// <param name="context">the context for the server-side call.</param>
        /// <exception>An exception is thrown if the digital twin client cannot perform an update.</exception>
        /// <returns>The response status of the update.</returns>
        public override async Task<UpdateDigitalTwinResponse> UpdateDigitalTwin(UpdateDigitalTwinRequest request, ServerCallContext context)
        {
            try
            {
                await _digitalTwinClient.UpdateDigitalTwinAsync(request.ModelId, request.InstanceId, request.InstancePropertyPath, request.Data);
            }
            catch (Exception ex)
            {
                _logger.LogError(ex.Message);
                throw;
            }

            return new UpdateDigitalTwinResponse
            {
                Reply = $"Successfully set instance {request.InstanceId}{request.InstancePropertyPath} based on model {request.ModelId} to {request.Data}"
            };
        }
    }
}

