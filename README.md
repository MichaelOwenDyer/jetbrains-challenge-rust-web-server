# Jetbrains Internship - Rust Web Application for Managing Test Data - Test Task 1

## Project Overview
This application allows users to create and view blog posts, complete with text, a publication date, an optional image, a username, and an optional user avatar.
The application persists blog posts in an SQLite database and stores images locally on the file system.

Here is the repository file structure:
```
.
├── assets                  # Static assets for the application
│   ├── main.css            # Stylesheet for the application
│   ├── favicon.ico         # Favicon for the application
├── images                  # Uploaded images are stored here
├── migrations              # Diesel database migrations; contains only initial schema
├── src
│   ├── main.rs             # Main entry point for the application; the server is started here
│   ├── model.rs            # Domain models
│   ├── client.rs           # Client web app written in Dioxus
│   ├── api.rs              # API routes for client-server interaction
│   ├── server
│   │   ├── mod.rs          # Server module
│   │   ├── images.rs       # Image handling utilities
│   │   ├── persistence
│   │   │   ├── mod.rs      # Persistence module
│   │   │   ├── schema.rs   # Database schema (generated by Diesel)
│   │   │   ├── database.rs # Database queries written with Diesel
├── README.md               # This file
├── Cargo.toml              # Cargo configuration file
├── diesel.toml             # Diesel configuration file
├── Dioxus.toml             # Dioxus configuration file
├── example.env             # Example environment file
├── Dockerfile              # Dockerfile for building the application

```

Before doing anything else, you should create a `.env` file and configure it:
```bash
cp example.env .env
```

The `DATABASE_URL` environment variable is required, while `HOST_ADDR` and `LOG_LEVEL` are optional
and default to `0.0.0.0:8080` and `INFO`, respectively.

To run the application with Docker, run the following commands:
```bash
docker build -t blogposts .
docker run -p 8080:8080 blogposts
```

Once running, the application can be accessed at `http://localhost:8080/home`.