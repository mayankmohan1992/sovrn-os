package main

import (
	"context"
	"encoding/json"
	"log"
	"net"
	"os"
	"os/signal"
	"syscall"

	"github.com/sovrn-os/sovrn-cdn-agent/internal/api"
	"github.com/sovrn-os/sovrn-cdn-agent/internal/config"
)

func main() {
	cfgPath := "/etc/sovrn/cdn-agent.toml"
	if len(os.Args) > 2 && (os.Args[1] == "--config" || os.Args[1] == "-c") {
		cfgPath = os.Args[2]
	}

	cfg, err := config.Load(cfgPath)
	if err != nil {
		log.Fatalf("Failed to load config: %v", err)
	}

	// Ensure directories exist
	os.MkdirAll(cfg.SocketsDir, 0755)
	os.MkdirAll(cfg.DataDir, 0755)
	os.MkdirAll(cfg.CacheDir, 0755)

	// Start JSON-RPC server on Unix socket
	socketPath := cfg.SocketsDir + "/cdn.sock"
	os.Remove(socketPath)

	listener, err := net.Listen("unix", socketPath)
	if err != nil {
		log.Fatalf("Failed to listen on %s: %v", socketPath, err)
	}
	defer listener.Close()

	handler := api.NewHandler(cfg)

	log.Printf("sovrn-cdn-agent starting on %s", socketPath)

	// Notify systemd
	sdNotify("READY=1")

	// Handle shutdown
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, syscall.SIGINT, syscall.SIGTERM)

	go func() {
		<-sigChan
		log.Println("Shutting down sovrn-cdn-agent")
		cancel()
		listener.Close()
	}()

	for {
		conn, err := listener.Accept()
		if err != nil {
			select {
			case <-ctx.Done():
				return
			default:
				log.Printf("Accept error: %v", err)
				continue
			}
		}
		go handler.HandleConnection(conn)
	}
}

func sdNotify(state string) {
	socket := os.Getenv("NOTIFY_SOCKET")
	if socket == "" {
		return
	}
	conn, err := net.Dial("unixgram", socket)
	if err != nil {
		return
	}
	defer conn.Close()
	conn.Write([]byte(state))
}

// JSONRPCRequest represents a JSON-RPC 2.0 request
type JSONRPCRequest struct {
	JSONRPC string          `json:"jsonrpc"`
	Method  string          `json:"method"`
	Params  json.RawMessage `json:"params"`
	ID      json.RawMessage `json:"id"`
}

// JSONRPCResponse represents a JSON-RPC 2.0 response
type JSONRPCResponse struct {
	JSONRPC string          `json:"jsonrpc"`
	Result  interface{}     `json:"result,omitempty"`
	Error   *JSONRPCError   `json:"error,omitempty"`
	ID      json.RawMessage `json:"id"`
}

type JSONRPCError struct {
	Code    int    `json:"code"`
	Message string `json:"message"`
}
