import socket

def send_request(request: str, host: str = "127.0.0.1", port: int = 7878):
    # Create a TCP/IP socket
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        # Connect to the daemon
        sock.connect((host, port))
        # Send the request encoded as bytes
        sock.sendall(request.encode())
        # Receive the response (up to 4096 bytes)
        response = sock.recv(4096)
        print("Response:", response.decode())

if __name__ == '__main__':
    # Test GET_HISTORY request
    print("Sending GET_HISTORY request...")
    send_request("GET_HISTORY")
    
    # Test CLEAR_HISTORY request
    # print("\nSending CLEAR_HISTORY request...")
    # send_request("CLEAR_HISTORY")
