# News Tracker (NT)

A news aggregator that uses AI to detect similar stories across many digital news publications, with a unique focus on detecting divergences in coverage between different sources.

## Design Goals

### Core Features
- **Cross-Source Analysis**: Detect and analyze how the same story is covered across different news sources
- **Divergence Detection**: Identify parts of stories that are present in some sources but missing in others
- **AI-Powered Analysis**: Use LLMs for summarization and semantic understanding of articles
- **Vector Search**: Utilize multiple vector databases for efficient similarity search and article comparison
- **Health Checks**: Automatic storage backend health verification with retries
- **Flexible Scraping**: Support for periodic scraping with customizable intervals
- **Article Sections**: Intelligent article section splitting and analysis
- **Similarity Scoring**: Cosine similarity-based article comparison
- **Author Detection**: Automatic author extraction from multiple sources

### Architecture
- **Modular Design**: Each major feature is a separate crate with clear responsibilities
- **Extensible**: Easy to add new news sources, AI models, and features
- **Country-Specific Organization**: News source scrapers are organized by country for better maintainability
- **CLI-First**: Command-line interface for all functionality, making it easy to script and automate
- **Clean Architecture**: Clear separation between web, service, and data layers

### Technology Stack
- **Web Scraping**: Using `scraper` for robust HTML parsing
- **LLM Integration**: Multiple model support:
  - Ollama (default)
  - DeepSeek API
  - LangChain integration
- **Vector Storage**: Multiple backend options:
  - In-memory storage (default)
  - ChromaDB for vector similarity search
  - Qdrant for vector similarity search
  - SQLite for persistent storage
- **Web Interface**: Using `axum` for the API server
- **Docker Support**: Containerized deployment with health checks
- **Rate Limiting**: Built-in concurrency control for API calls

### Storage Backends
The project supports multiple storage backends that can be enabled via feature flags:

```bash
# Build with all backends enabled
cargo build --release --features "chroma qdrant sqlite"

# Build with specific backends
cargo build --release --features "chroma"  # Only ChromaDB
cargo build --release --features "qdrant"  # Only Qdrant
cargo build --release --features "sqlite"  # Only SQLite
```

When running the CLI, specify the backend using the `--storage` flag and configure URLs using `--model-url` and `--backend-url`:

```bash
# Use in-memory storage (default)
nt scrapers list

# Use Qdrant with custom URLs
nt --storage qdrant --model-url http://ollama:2543 --backend-url http://qdrant:3244 scrapers list

# Use ChromaDB with custom URLs
nt --storage chroma --model-url http://ollama:2543 --backend-url http://chroma:8000 scrapers list

# Use SQLite with custom model URL
nt --storage sqlite --model-url http://ollama:2543 scrapers list
```

Requirements for each backend:
- **ChromaDB**: Requires ChromaDB server running on `http://localhost:8000` (or custom URL via `--backend-url`)
- **Qdrant**: Requires Qdrant server running on `http://localhost:6333` (or custom URL via `--backend-url`)
- **SQLite**: Creates a database file at `./articles.db` in the current working directory
- **Ollama**: Requires Ollama server running on `http://localhost:11434` (or custom URL via `--model-url`)

### Configuration Options
- `--storage`: Choose the storage backend (memory, chroma, qdrant, sqlite)
- `--model-url`: URL for the model server (default: http://localhost:11434)
- `--backend-url`: URL for the vector storage backend (default: depends on backend)
- `--model`: Choose the inference model (ollama, deepseek)
- `--interval`: Set periodic scraping interval (e.g., 1h, 30m, 1d, 1h15m30s)

### Current Crates
- `nt_core`: Core types, utilities, and CLI entry point
- `nt_scrappers`: News source scraping functionality
- `nt_inference`: AI/LLM integration for analysis
  - Manages LLM interactions through DeepSeek API
  - Handles text embeddings and similarity calculations
  - Implements the divergence detection algorithm
- `nt_web`: Web interface for viewing results

## Project Status

### Implemented Features
- [x] Basic project structure and crate organization
- [x] CLI framework with subcommands
- [x] Argentine news source scrapers:
  - [x] Clarín
  - [x] La Nación
  - [x] La Voz
- [x] Article parsing and section splitting
- [x] Basic test infrastructure
- [x] Article status tracking (new, updated, unchanged)
- [x] Multiple vector database integrations:
  - [x] ChromaDB
  - [x] Qdrant
  - [x] SQLite
- [x] Multiple LLM backend support:
  - [x] Ollama
  - [x] DeepSeek
  - [x] LangChain
- [x] Web API structure with proper layer separation
- [x] Docker support with health checks
- [x] Periodic scraping with customizable intervals
- [x] Article similarity scoring
- [x] Author detection from multiple sources
- [x] Storage backend health verification
- [x] Rate limiting for API calls

### In Progress
- [ ] Article divergence algorithm implementation
- [ ] Web interface development
- [ ] Article comparison metrics
- [ ] Rate limiting and caching for scrapers

### Planned Features
- [ ] More news sources from different countries
- [ ] Advanced article comparison metrics
- [ ] Real-time news monitoring
- [ ] API for third-party integrations
- [ ] Automated testing with mock HTTP responses
- [ ] Support for multiple LLM backends:
  - [ ] OpenAI
  - [ ] Local models via llama.cpp

## Usage

### Installation
```bash
# Clone the repository
git clone https://github.com/yourusername/NT.git
cd NT

# Build the project
cargo build --release
```

### CLI Commands
```bash
# List available scrapers
nt scrapers list

# Scrape articles from all sources (default behavior)
nt scrapers scrape

# Scrape articles from a specific country
nt scrapers scrape source argentina

# Scrape articles from a specific source
nt scrapers scrape source argentina/clarin

# Scrape a specific article
nt scrapers scrape url https://www.lanacion.com.ar/some-article

# Run periodic scraping with custom interval
nt scrapers scrape source --interval 1h    # Scrape every hour
nt scrapers scrape source --interval 30m   # Scrape every 30 minutes
nt scrapers scrape source --interval 1d    # Scrape every day
nt scrapers scrape source --interval 1h15m # Scrape every 1 hour and 15 minutes
nt scrapers scrape source --interval 1h15m30s # Scrape every 1 hour, 15 minutes and 30 seconds

# Use specific model and storage backend
nt --model deepseek --storage chroma scrapers scrape
nt --model ollama --storage qdrant scrapers scrape
nt --model langchain --storage sqlite scrapers scrape
```

### Web API
```bash
# Start the web server
nt web serve
```

Available endpoints:
- `GET /api/articles` - List all articles
- `POST /api/articles` - Create a new article
- `GET /api/articles/:id` - Get a specific article
- `GET /api/articles/:id/similar` - Find similar articles
- `GET /api/articles/:id/divergence` - Get article divergence analysis

## Development

### Docker Compose

You can run the application with either Qdrant or Chroma as the storage backend using Docker Compose:

#### With Qdrant
```bash
# Start the application with Qdrant backend
docker compose -f docker-compose.qdrant.yml up app_qdrant qdrant

# Or run in detached mode
docker compose -f docker-compose.qdrant.yml up -d app_qdrant qdrant
```

#### With Chroma
```bash
# Start the application with Chroma backend
docker compose -f docker-compose.chroma.yml up app_chroma chroma

# Or run in detached mode
docker compose -f docker-compose.chroma.yml up -d app_chroma chroma
```

The services will be available at:
- Qdrant: http://localhost:6333
- Chroma: http://localhost:8000

### Environment Variables
- `SCRAPE_INTERVAL`: Interval between scraping cycles (default: 3600)
- `STORAGE`: Storage backend to use (default: sqlite)
- `MODEL_URL`: URL for the model server
- `BACKEND_URL`: URL for the vector storage backend
- `RUST_LOG`: Logging level (default: info)

### Health Checks
The application includes automatic health checks for:
- Storage backend connectivity
- Model server availability
- Database migrations
- API endpoints

### Rate Limiting
Built-in rate limiting is implemented for:
- API calls to external services
- Database operations
- Model inference requests