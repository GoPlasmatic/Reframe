import React from 'react';
import ReactDOM from 'react-dom/client';
import { MantineProvider, createTheme } from '@mantine/core';
import '@mantine/core/styles.css';
import '@mantine/notifications/styles.css';
import './plasmatic-theme.css';
import App from './App';

const theme = createTheme({
  primaryColor: 'emerald',
  colors: {
    emerald: [
      '#e6ffef',
      '#b3ffd1',
      '#80ffb3',
      '#4dff95',
      '#1aff77',
      '#00ff59',
      '#00e64d',
      '#00cc42',
      '#00b337',
      '#00992c'
    ],
    'midnight-green': [
      '#f0f2f4',
      '#d1d7dd',
      '#b3bcc6',
      '#94a1af',
      '#758698',
      '#566b81',
      '#3d5066',
      '#2a3a52',
      '#1a2332',
      '#0f1420'
    ],
    amaranth: [
      '#fce4ec',
      '#f8bbd9',
      '#f48fb1',
      '#f06292',
      '#ec407a',
      '#e91e63',
      '#d81b60',
      '#c2185b',
      '#ad1457',
      '#880e4f'
    ],
    'sun-glow': [
      '#fff8e1',
      '#ffecb3',
      '#ffe082',
      '#ffd54f',
      '#ffca28',
      '#ffc107',
      '#ffb300',
      '#ffa000',
      '#ff8f00',
      '#ff6f00'
    ],
    'blue-green': [
      '#e0f2f1',
      '#b2dfdb',
      '#80cbc4',
      '#4db6ac',
      '#26a69a',
      '#009688',
      '#00897b',
      '#00796b',
      '#00695c',
      '#004d40'
    ]
  },
  fontFamily: '"DM Sans", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
  headings: {
    fontFamily: '"Space Grotesk", -apple-system, BlinkMacSystemFont, sans-serif',
  },
  defaultRadius: 'md',
  other: {
    plasmaticColors: {
      midnightGreen: '#1a2332',
      emerald: '#00ff87',
      amaranth: '#e91e63',
      sunGlow: '#ffab00',
      blueGreen: '#00bcd4'
    }
  }
});

const root = ReactDOM.createRoot(document.getElementById('root'));
root.render(
  <React.StrictMode>
    <MantineProvider theme={theme}>
      <App />
    </MantineProvider>
  </React.StrictMode>
); 