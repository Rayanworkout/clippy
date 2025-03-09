#!/usr/bin/env python3
import socket

TCP_IP = "127.0.0.1"
TCP_PORT = 7878  # Replace with your TCP port if different
BUFFER_SIZE = 1024  # Adjust buffer size as needed


def send_request(request: str) -> str:
    """Send a request string to the endpoint and return the response."""
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.connect((TCP_IP, TCP_PORT))
        # Send the request encoded as bytes
        sock.sendall(request.encode())
        # Receive the response (blocking until some data is received)
        response = sock.recv(BUFFER_SIZE)
    return response.decode()


if __name__ == "__main__":
    response = send_request("GET_HISTORY")
    print("Response:", response)

    # response = send_request("RESET_HISTORY")
    # print("Response:", response)
