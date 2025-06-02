# Reframe Web UI

A modern Material-UI web interface for testing the Reframe SWIFT MT103 to ISO 20022 converter API.

## Features

- 🎨 Modern Material-UI design
- 📝 Large text area for SWIFT MT103 input
- 🔄 Real-time API integration
- 🎯 XML syntax highlighting
- 📱 Responsive design
- ⚡ Fast and lightweight

## Live Demo

The web UI is deployed at: [https://GoPlasmatic.github.io/Reframe](https://GoPlasmatic.github.io/Reframe)

## Development

### Prerequisites

- Node.js 18+ installed
- npm package manager

### Installation

1. Navigate to the web-ui directory:
```bash
cd web-ui
```

2. Install dependencies:
```bash
npm install
```

3. Start the development server:
```bash
npm start
```

4. Open [http://localhost:3000](http://localhost:3000) to view it in the browser.

### Building for Production

```bash
npm run build
```

This builds the app for production to the `build` folder.

### Deployment

The web UI is automatically deployed to GitHub Pages when changes are pushed to the main branch. The deployment is handled by the GitHub Actions workflow in `.github/workflows/deploy-web-ui.yml`.

## API Integration

The web UI connects to the Reframe API deployed on Azure Container Instances:
- **Production API**: `http://reframe-api-prod.eastus.azurecontainer.io:3000/reframe`

### CORS Configuration

Note: The API must be configured to allow cross-origin requests from the GitHub Pages domain. If you encounter CORS issues, you may need to:

1. Add CORS headers to the Rust API
2. Use a proxy server
3. Run the web UI locally for development

## Usage

1. Paste a SWIFT MT103 message in the left text area
2. Click the "Transform" button
3. View the converted ISO 20022 pacs.008.001.13 XML in the right panel with syntax highlighting

### Sample MT103 Message

The UI includes a "Load Sample MT103" button that loads a pre-configured sample message for testing.

## Technologies Used

- **React** 18.2.0 - UI framework
- **Material-UI** 5.15.15 - Component library
- **react-syntax-highlighter** - XML syntax highlighting
- **Create React App** - Build tooling

## License

This project is part of the Reframe SWIFT to ISO 20022 converter and is licensed under the Apache License. 