// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

using Azure.DigitalTwins.Core;
using Moq;
using Microsoft.Extensions.Logging;
using NUnit.Framework;

namespace Microsoft.ESDV.CloudConnector.Azure.Tests
{
    [TestFixture]
    public class MQTTConnectorTests
    {
        private MQTTConnector _connector;

        [SetUp]
        public void Setup()
        {
            var mockLogger = new Mock<ILogger>();
            var mockDigitalTwinsClient = new Mock<DigitalTwinsClient>();
            _connector = new MQTTConnector(mockDigitalTwinsClient.Object, mockLogger.Object);
        }

        [Test]
        public async Task UpdateDigitalTwinAsync_ShouldSucceed()
        {
            const string modelID = "some-model";
            const string instanceID = "some-instance";
            const string instancePropertyPath = "some-instance-property";
            await _connector.UpdateDigitalTwinAsync(modelID, instanceID, instancePropertyPath, "44.5");
            Assert.Pass();
        }

        [Test]
        public void UpdateDigitalTwinAsync_ThrowNotSupported()
        {
            const string modelID = "some-model";
            const string instanceID = "some-instance";
            const string instancePropertyPath = "some-instance-property";
            Assert.ThrowsAsync<NotSupportedException>(async () => await _connector.UpdateDigitalTwinAsync(modelID, instanceID, instancePropertyPath, "test1234"));
            Assert.ThrowsAsync<NotSupportedException>(async () => await _connector.UpdateDigitalTwinAsync(modelID, instanceID, instancePropertyPath, "1234test"));
            Assert.ThrowsAsync<NotSupportedException>(async () => await _connector.UpdateDigitalTwinAsync(modelID, instanceID, instancePropertyPath, ""));
        }
    }
}
