# Reframe Web UI

This is the React-based web interface for the Reframe SWIFT to ISO 20022 converter. Built with Mantine UI 8.1.0 and featuring a modern, responsive design powered by Vite.

## Features

- 🎨 **Mantine UI 8.1.0**: Modern React components library with excellent performance
- 🚀 **Auto-Detection**: Automatically detects SWIFT message type and processing method
- 📱 **Responsive Design**: Works seamlessly on desktop and mobile devices  
- 🎯 **Sample Loading**: Auto-loaded sample messages for supported MT103 transformations
- ✨ **Syntax Highlighting**: XML output with beautiful syntax coloring using Prism
- 🔄 **Real-time Feedback**: Comprehensive error handling and success notifications
- 🎭 **Split Panel Layout**: Side-by-side input and output for easy comparison
- 📊 **Progress Indicators**: Visual feedback during transformation processing
- 🔢 **Multi-Message Support**: Handles both single and multiple message transformations

## Technology Stack

- **React** 19.1.0 (latest stable)
- **Mantine Core** 8.1.0 (modern UI components)
- **Mantine Hooks** 8.1.0 (utility hooks)
- **Mantine Notifications** 8.1.0 (toast notifications)
- **Tabler Icons React** 3.34.0 (icon library)
- **React Syntax Highlighter** 15.6.1 (code highlighting)
- **Vite** 6.3.5 (fast build tool and dev server)

## Recent Updates

### Major Framework Migration (January 2025)
- ✅ **Migrated from Material UI to Mantine UI 8.1.0** - Modern, lightweight alternative
- ✅ **Upgraded to React 19.1.0** - Latest React with improved performance
- ✅ **Switched from Create React App to Vite** - Faster development and build times
- ✅ **Enhanced Error Handling** - Comprehensive error display and processing info
- ✅ **Multi-Message Support** - Handles multiple SWIFT messages in single request
- ✅ **Improved UI/UX** - Better visual feedback and responsive design

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
# or
npm run dev
```

4. Open [http://localhost:3000](http://localhost:3000) to view it in the browser.

### Building for Production

```bash
npm run build
```

This builds the app for production to the `build` folder using Vite's optimized build process.

### Deployment

The web UI is automatically deployed to GitHub Pages when changes are pushed to the main branch. The deployment is handled by the GitHub Actions workflow and uses Vite's build system.

## API Integration

The web UI connects to the Reframe API with intelligent endpoint detection:

- **Development**: Proxies to `http://localhost:8000/reframe` via Vite dev server
- **Production**: Uses relative URL `/reframe` served from the same domain

### API Configuration

The Vite configuration includes a development proxy that forwards API requests to the local Rust server:

```javascript
server: {
  proxy: {
    '/reframe': {
      target: 'http://localhost:8000',
      changeOrigin: true,
      secure: false,
    },
  },
}
```

## Supported Transformations

Currently supports comprehensive MT103 processing:

| SWIFT Message | ISO 20022 Output | Description |
|---------------|------------------|-------------|
| **MT103** | pacs.008.001.08 | Customer Credit Transfer (all variants) |

### MT103 Variants Supported
- **Normal Processing**: Standard credit transfers
- **STP (Straight Through Processing)**: Automated processing
- **Rejection Processing**: Handling of rejected transfers
- **Return Processing**: Processing returned transfers

## Usage

1. The interface automatically loads a sample MT103 message
2. Modify the message or paste your own SWIFT MT103 message
3. Click the "Transform" button
4. View the converted ISO 20022 pacs.008.001.08 XML with syntax highlighting
5. Processing information and message counts are displayed for transparency

### Sample MT103 Message

The UI includes an auto-loaded comprehensive sample MT103 message that demonstrates the full transformation capabilities.

## Error Handling

The application provides comprehensive error handling:
- **Network Errors**: Connection issues with the API
- **Validation Errors**: Invalid SWIFT message format
- **Processing Errors**: Transformation failures with detailed messages
- **JSON Parsing**: Response format validation

## Build System

- **Vite**: Fast build tool with HMR (Hot Module Replacement)
- **ESBuild**: Ultra-fast JavaScript bundler
- **JSX Support**: Automatic JSX transformation for `.js` files
- **Source Maps**: Available in production builds for debugging
- **GitHub Pages**: Optimized deployment with correct base path

## License

This project is part of the Reframe SWIFT to ISO 20022 converter and is licensed under the Apache License. 