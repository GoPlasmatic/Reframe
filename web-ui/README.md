# Reframe Web UI

This is the React-based web interface for the Reframe SWIFT to ISO 20022 converter. Built with Material UI v7.1.1 and featuring a modern, responsive design.

## Features

- 🎨 **Material UI v7.1.1**: Latest version with improved ESM support and performance
- 🚀 **Auto-Detection**: Automatically detects SWIFT message type
- 📱 **Responsive Design**: Works seamlessly on desktop and mobile devices  
- 🎯 **Sample Loading**: One-click sample message loading for all supported types
- ✨ **Syntax Highlighting**: XML output with beautiful syntax coloring
- 🔄 **Real-time Feedback**: Inline success/error messages with Material icons
- 🎭 **Split Panel Layout**: Side-by-side input and output for easy comparison

## Technology Stack

- **React** 18.3.1
- **Material UI** 7.1.1 (latest stable release)
- **Material Icons** 7.1.1
- **React Syntax Highlighter** 15.6.1
- **Emotion** for styling (11.13.5)

## Recent Updates

### Material UI v7.1.1 Upgrade (January 2025)
- ✅ **Updated from v5.15.15 to v7.1.1** - Latest stable release
- ✅ **Improved ESM Support** - Better compatibility with modern bundlers
- ✅ **Enhanced Performance** - Optimized bundle size and loading times
- ✅ **Future-Ready** - Prepared for upcoming Material Design 3 updates
- ✅ **Backward Compatibility** - No breaking changes required in existing code

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