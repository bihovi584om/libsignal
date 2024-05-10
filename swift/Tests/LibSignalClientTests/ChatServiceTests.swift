//
// Copyright 2024 Signal Messenger, LLC.
// SPDX-License-Identifier: AGPL-3.0-only
//

import Foundation
@testable import LibSignalClient
import SignalFfi
import XCTest

extension ChatService {
    func injectServerRequest(base64: String) {
        self.injectServerRequest(Data(base64Encoded: base64)!)
    }

    func injectServerRequest(_ requestBytes: Data) {
        withNativeHandle { handle in
            requestBytes.withUnsafeBorrowedBuffer { requestBytes in
                failOnError(signal_testing_chat_service_inject_raw_server_request(handle, requestBytes))
            }
        }
    }
}

final class ChatServiceTests: TestCaseBase {
// These testing endpoints aren't generated in device builds, to save on code size.
#if !os(iOS) || targetEnvironment(simulator)

    private static let userAgent = "test"
    private static let expectedStatus: UInt16 = 200
    private static let expectedMessage = "OK"
    private static let expectedContent = "content".data(using: .utf8)
    private static let expectedHeaders = ["content-type": "application/octet-stream", "forwarded": "1.1.1.1"]

    func testConvertResponse() throws {
        do {
            // Empty body
            var rawResponse = SignalFfiChatResponse()
            try checkError(signal_testing_chat_service_response_convert(&rawResponse, false))
            let response = try ChatService.Response(consuming: rawResponse)
            XCTAssertEqual(Self.expectedStatus, response.status)
            XCTAssertEqual(Self.expectedMessage, response.message)
            XCTAssertEqual(Self.expectedHeaders, response.headers)
            XCTAssert(response.body.isEmpty)
        }

        do {
            // Present body
            var rawResponse = SignalFfiChatResponse()
            try checkError(signal_testing_chat_service_response_convert(&rawResponse, true))
            let response = try ChatService.Response(consuming: rawResponse)
            XCTAssertEqual(Self.expectedStatus, response.status)
            XCTAssertEqual(Self.expectedMessage, response.message)
            XCTAssertEqual(Self.expectedHeaders, response.headers)
            XCTAssertEqual(Self.expectedContent, response.body)
        }
    }

    func testConvertDebugInfo() throws {
        var rawDebugInfo = SignalFfiChatServiceDebugInfo()
        try checkError(signal_testing_chat_service_debug_info_convert(&rawDebugInfo))
        let debugInfo = ChatService.DebugInfo(consuming: rawDebugInfo)
        XCTAssertEqual(2, debugInfo.reconnectCount)
        XCTAssertEqual(.ipv4, debugInfo.ipType)
        XCTAssertEqual(0.2, debugInfo.duration)
        XCTAssertEqual("connection_info", debugInfo.connectionInfo)
    }

    func testConvertResponseAndDebugInfo() throws {
        var rawResponseAndDebugInfo = SignalFfiResponseAndDebugInfo()
        try checkError(signal_testing_chat_service_response_and_debug_info_convert(&rawResponseAndDebugInfo))

        let response = try ChatService.Response(consuming: rawResponseAndDebugInfo.response)
        XCTAssertEqual(Self.expectedStatus, response.status)
        XCTAssertEqual(Self.expectedMessage, response.message)
        XCTAssertEqual(Self.expectedHeaders, response.headers)
        XCTAssertEqual(Self.expectedContent, response.body)

        let debugInfo = ChatService.DebugInfo(consuming: rawResponseAndDebugInfo.debug_info)
        XCTAssertEqual(2, debugInfo.reconnectCount)
        XCTAssertEqual(.ipv4, debugInfo.ipType)
        XCTAssertEqual(0.2, debugInfo.duration)
        XCTAssertEqual("connection_info", debugInfo.connectionInfo)
    }

    func testConvertError() throws {
        do {
            try checkError(signal_testing_chat_service_error_convert())
            XCTFail("error not thrown")
        } catch SignalError.connectionTimeoutError(_) {
            // Okay
        }
        do {
            try checkError(signal_testing_chat_service_inactive_error_convert())
            XCTFail("error not thrown")
        } catch SignalError.chatServiceInactive(_) {
            // Okay
        }
    }

    func testConstructRequest() throws {
        let expectedMethod = "GET"
        let expectedPathAndQuery = "/test"

        let request = ChatService.Request(method: expectedMethod, pathAndQuery: expectedPathAndQuery, headers: Self.expectedHeaders, body: Self.expectedContent, timeout: 5)
        let internalRequest = try ChatService.InternalRequest(request)
        try internalRequest.withNativeHandle { internalRequest in
            XCTAssertEqual(expectedMethod, try invokeFnReturningString {
                signal_testing_chat_request_get_method($0, internalRequest)
            })
            XCTAssertEqual(expectedPathAndQuery, try invokeFnReturningString {
                signal_testing_chat_request_get_path($0, internalRequest)
            })
            XCTAssertEqual(Self.expectedContent, try invokeFnReturningData {
                signal_testing_chat_request_get_body($0, internalRequest)
            })
            for (k, v) in Self.expectedHeaders {
                XCTAssertEqual(v, try invokeFnReturningString {
                    signal_testing_chat_request_get_header_value($0, internalRequest, k)
                })
            }
        }
    }

    func testListenerCallbacks() throws {
        class Listener: ChatListener {
            var stage = 0
            let queueEmpty: XCTestExpectation
            let firstMessageReceived: XCTestExpectation
            let secondMessageReceived: XCTestExpectation

            init(queueEmpty: XCTestExpectation, firstMessageReceived: XCTestExpectation, secondMessageReceived: XCTestExpectation) {
                self.queueEmpty = queueEmpty
                self.firstMessageReceived = firstMessageReceived
                self.secondMessageReceived = secondMessageReceived
            }

            func chatService(_ chat: ChatService, didReceiveIncomingMessage envelope: Data, serverDeliveryTimestamp: UInt64, sendAck: () async throws -> Void) {
                // This assumes a little-endian platform.
                XCTAssertEqual(envelope, withUnsafeBytes(of: serverDeliveryTimestamp) { Data($0) })
                switch serverDeliveryTimestamp {
                case 1000:
                    XCTAssertEqual(self.stage, 0)
                    self.stage += 1
                    self.firstMessageReceived.fulfill()
                case 2000:
                    XCTAssertEqual(self.stage, 1)
                    self.stage += 1
                    self.secondMessageReceived.fulfill()
                default:
                    XCTFail("unexpected message")
                }
            }

            func chatServiceDidReceiveQueueEmpty(_: ChatService) {
                XCTAssertEqual(self.stage, 2)
                self.stage += 1
                self.queueEmpty.fulfill()
            }
        }

        let net = Net(env: .staging, userAgent: Self.userAgent)
        let chat = net.createChatService(username: "", password: "")
        let listener = Listener(
            queueEmpty: expectation(description: "queue empty"),
            firstMessageReceived: expectation(description: "first message received"),
            secondMessageReceived: expectation(description: "second message received")
        )
        chat.setListener(listener)

        // The following payloads were generated via protoscope.
        // % protoscope -s | base64
        // The fields are described by chat_websocket.proto in the libsignal-net crate.

        // 1: {"PUT"}
        // 2: {"/api/v1/message"}
        // 3: {1000i64}
        // 5: {"x-signal-timestamp:1000"}
        // 4: 1
        chat.injectServerRequest(base64: "CgNQVVQSDy9hcGkvdjEvbWVzc2FnZRoI6AMAAAAAAAAqF3gtc2lnbmFsLXRpbWVzdGFtcDoxMDAwIAE=")
        // 1: {"PUT"}
        // 2: {"/api/v1/message"}
        // 3: {2000i64}
        // 5: {"x-signal-timestamp:2000"}
        // 4: 2
        chat.injectServerRequest(base64: "CgNQVVQSDy9hcGkvdjEvbWVzc2FnZRoI0AcAAAAAAAAqF3gtc2lnbmFsLXRpbWVzdGFtcDoyMDAwIAI=")

        // Sending an invalid message should not affect the listener at all, nor should it stop future requests.
        // 1: {"PUT"}
        // 2: {"/invalid"}
        // 4: 10
        chat.injectServerRequest(base64: "CgNQVVQSCC9pbnZhbGlkIAo=")

        // 1: {"PUT"}
        // 2: {"/api/v1/queue/empty"}
        // 4: 99
        chat.injectServerRequest(base64: "CgNQVVQSEy9hcGkvdjEvcXVldWUvZW1wdHkgYw==")

        waitForExpectations(timeout: 2)
        XCTAssertEqual(listener.stage, 3)
    }

#endif

    func testListenerCleanup() throws {
        class Listener: ChatListener {
            let expectation: XCTestExpectation
            init(expectation: XCTestExpectation) {
                self.expectation = expectation
            }

            deinit {
                expectation.fulfill()
            }

            func chatServiceDidReceiveQueueEmpty(_: ChatService) {}
            func chatService(_ chat: ChatService, didReceiveIncomingMessage envelope: Data, serverDeliveryTimestamp: UInt64, sendAck: () async throws -> Void) {}
        }

        let net = Net(env: .staging, userAgent: Self.userAgent)

        do {
            let chat = net.createChatService(username: "", password: "")

            do {
                let listener = Listener(expectation: expectation(description: "first listener destroyed"))
                chat.setListener(listener)
            }
            do {
                let listener = Listener(expectation: expectation(description: "second listener destroyed"))
                chat.setListener(listener)
            }
            // Clearing the listener has a separate implementation, so let's make sure both get destroyed.
            chat.setListener(nil)
            waitForExpectations(timeout: 2)

            do {
                let listener = Listener(expectation: expectation(description: "third listener destroyed"))
                chat.setListener(listener)
            }
        }
        // If we destroy the ChatService, we should also clean up the listener.
        waitForExpectations(timeout: 2)
    }

    func testConnectUnauth() async throws {
        // Use the presence of the proxy server environment setting to know whether we should make network requests in our tests.
        guard ProcessInfo.processInfo.environment["LIBSIGNAL_TESTING_PROXY_SERVER"] != nil else {
            throw XCTSkip()
        }

        let net = Net(env: .staging, userAgent: Self.userAgent)
        let chat = net.createChatService(username: "", password: "")
        // Just make sure we can connect.
        try await chat.connectUnauthenticated()
        try await chat.disconnect()
    }

    func testConnectUnauthThroughProxy() async throws {
        guard let PROXY_SERVER = ProcessInfo.processInfo.environment["LIBSIGNAL_TESTING_PROXY_SERVER"] else {
            throw XCTSkip()
        }

        // The default TLS proxy config doesn't support staging, so we connect to production.
        let net = Net(env: .production, userAgent: Self.userAgent)
        let host: Substring
        let port: UInt16
        if let colonIndex = PROXY_SERVER.firstIndex(of: ":") {
            host = PROXY_SERVER[..<colonIndex]
            port = UInt16(PROXY_SERVER[colonIndex...].dropFirst())!
        } else {
            host = PROXY_SERVER[...]
            port = 443
        }
        try net.setProxy(host: String(host), port: port)

        let chat = net.createChatService(username: "", password: "")
        // Just make sure we can connect.
        try await chat.connectUnauthenticated()
        try await chat.disconnect()
    }

    func testConnectFailsWithInvalidProxy() async throws {
        // The default TLS proxy config doesn't support staging, so we connect to production.
        let net = Net(env: .production, userAgent: Self.userAgent)
        do {
            try net.setProxy(host: "signalfoundation.org", port: 0)
            XCTFail("should not allow setting invalid proxy")
        } catch SignalError.ioError {
            // Okay
        }

        let chat = net.createChatService(username: "", password: "")
        // Make sure we *can't* connect.
        do {
            try await chat.connectUnauthenticated()
            XCTFail("should not allow connecting")
        } catch SignalError.connectionFailed {
            // Okay
        }
    }
}
