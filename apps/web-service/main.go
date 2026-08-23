package main

import (
	"log"
	"net/http"
	"os"

	"github.com/wiradigitalid/snapdown/apps/web-service/internal/server"
	"github.com/wiradigitalid/snapdown/apps/web-service/internal/store"
)

func main() {
	port := os.Getenv("PORT")
	if port == "" {
		port = "8080"
	}

	dataDir := os.Getenv("DATA_DIR")
	if dataDir == "" {
		dataDir = "./data"
	}

	dbPath := os.Getenv("DB_PATH")
	if dbPath == "" {
		dbPath = "./data/publication.db"
	}

	publishKey := os.Getenv("PUBLISH_KEY")

	st, err := store.Open(dbPath, dataDir)
	if err != nil {
		log.Fatalf("Failed to open store: %v", err)
	}
	defer st.Close()

	srv := server.New(st, publishKey)

	log.Printf("Starting Snapdown web-service on port %s...", port)
	if err := http.ListenAndServe(":"+port, srv.Router()); err != nil {
		log.Fatalf("Server failed: %v", err)
	}
}
