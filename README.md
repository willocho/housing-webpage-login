# housing-webpage-login

This application consists of a simple web app and server. The server is written in Rust, using the rocket and sqlx packages. The client is a react application written in typescript, built using vite.

## Setting Up the Connection to the Postgres Database
Before you are able to run the server, you will need to create a `.env` file in the server directory with the environment variable `DATABASE_URL` set to a valid postgres connection string using your username and password.

An example: `DATABASE_URL='postgresql://<your username>:<your password>@madison-data.house:5432`

## Client Commands

### Building the Client
Run `npm install`, then build the client using the `npm run build` command.

### Running a Local Dev Server
You can run a local dev server using the `npm run dev` command. The dev command is set up to proxy api requests to `http://localhost:8000` (see `webpage/vite.config.ts`, so you want to make sure that the application server is also running.

## Server Commands

### Running the server
You can run the application server using the `cargo run` command.
