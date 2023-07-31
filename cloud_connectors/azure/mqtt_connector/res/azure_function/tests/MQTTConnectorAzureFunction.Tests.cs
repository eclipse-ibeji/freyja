// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

using Azure.DigitalTwins.Core;
using Microsoft.Extensions.Logging;
using Moq;
using NUnit.Framework;
namespace Microsoft.ESDV.CloudConnector.Azure.Tests
{
    [TestFixture]
    public class MQTTConnectorAzureFunctionTests
    {
        private DigitalTwinsClient _client;
        private DigitalTwinsInstance _instance;
        private MQTTConnectorAzureFunction _connector;

        [SetUp]
        public void Setup()
        {
            _client = new Mock<DigitalTwinsClient>().Object;
            _connector = new MQTTConnectorAzureFunction(new Mock<ILogger<MQTTConnectorAzureFunction>>().Object);
            _instance = new DigitalTwinsInstance
            {
                model_id = "some-model",
                instance_id = "some-instance",
                instance_property_path = "some-instance-property",
                data = null
            };
        }

        [Test]
        public void ConvertStringToDataType_ShouldSucceed()
        {
            Assert.That(_connector.GetDataTypeFromString("int"), Is.EqualTo(typeof(System.Int32)));
            Assert.That(_connector.GetDataTypeFromString("double"), Is.EqualTo(typeof(System.Double)));
            Assert.That(_connector.GetDataTypeFromString("boolean"), Is.EqualTo(typeof(System.Boolean)));
            Assert.Throws<NotSupportedException>(() => _connector.GetDataTypeFromString("invalid-converter"));
        }

        [Test]
        public async Task UpdateDigitalTwinAsync_ShouldSucceed()
        {
            _instance.data = "44.5";
            await _connector.UpdateDigitalTwinAsync(_client, _instance, "double");
            Assert.Pass();

            _instance.data = "44";
            await _connector.UpdateDigitalTwinAsync(_client, _instance, "int");
            Assert.Pass();

            _instance.data = "true";
            await _connector.UpdateDigitalTwinAsync(_client, _instance, "boolean");
            Assert.Pass();
        }

        [Test]
        public void UpdateDigitalTwinAsync_ThrowNotSupported()
        {
            _instance.data = null;
            Assert.ThrowsAsync<NotSupportedException>(async () => await _connector.UpdateDigitalTwinAsync(_client, _instance));

            _instance.data = "test1234";
            Assert.ThrowsAsync<NotSupportedException>(async () => await _connector.UpdateDigitalTwinAsync(_client, _instance, "invalid-converter"));

            _instance.data = "";
            Assert.ThrowsAsync<NotSupportedException>(async () => await _connector.UpdateDigitalTwinAsync(_client, _instance, "double"));
        }
    }
}
