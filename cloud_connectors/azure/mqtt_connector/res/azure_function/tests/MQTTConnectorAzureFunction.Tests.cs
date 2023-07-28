// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

using Azure.DigitalTwins.Core;
using Moq;
using NUnit.Framework;

namespace Microsoft.ESDV.CloudConnector.Azure.Tests
{
    [TestFixture]
    public class MQTTConnectorAzureFunctionTests
    {
        private DigitalTwinsClient _client;
        private DigitalTwinsInstance _instance;

        [SetUp]
        public void Setup()
        {
            _client = new Mock<DigitalTwinsClient>().Object;
            _instance = new DigitalTwinsInstance
            {
                model_id = "some-model",
                instance_id = "some-instance",
                instance_property_path = "some-instance-property",
                data = null
            };
        }

        [Test]
        public async Task UpdateDigitalTwinAsync_ShouldSucceed()
        {
            _instance.data = "44.5";
            await MQTTConnectorAzureFunction.UpdateDigitalTwinAsync(_client, _instance);
            Assert.Pass();
        }

        [Test]
        public void UpdateDigitalTwinAsync_ThrowNotSupported()
        {
            Assert.ThrowsAsync<NotSupportedException>(async () => await MQTTConnectorAzureFunction.UpdateDigitalTwinAsync(_client, _instance));

            _instance.data = "test1234";
            Assert.ThrowsAsync<NotSupportedException>(async () => await MQTTConnectorAzureFunction.UpdateDigitalTwinAsync(_client, _instance));

            _instance.data = "1234test";
            Assert.ThrowsAsync<NotSupportedException>(async () => await MQTTConnectorAzureFunction.UpdateDigitalTwinAsync(_client, _instance));

            _instance.data = "";
            Assert.ThrowsAsync<NotSupportedException>(async () => await MQTTConnectorAzureFunction.UpdateDigitalTwinAsync(_client, _instance));
        }
    }
}
