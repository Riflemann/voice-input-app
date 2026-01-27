# Voice Input App

Voice Input App is a cross-platform desktop application designed for voice input, audio processing, and speech recognition. It leverages Tauri for the backend (written in Rust) and React with TypeScript for the frontend, providing a seamless and efficient user experience.

## Features

- **Voice Input**: Record and process audio input directly from your device.
- **Speech Recognition**: Convert spoken words into text using advanced recognition algorithms.
- **Customizable Settings**: Adjust audio and recognition settings to suit your needs.
- **Cross-Platform**: Runs on Windows, macOS, and Linux.
- **Modern UI**: Built with React, Zustand, and Tailwind CSS for a responsive and user-friendly interface.

## Project Structure

### Frontend
- **Framework**: React with TypeScript
- **State Management**: Zustand
- **Styling**: Tailwind CSS
- **Build Tool**: Vite
- **Directory**: `src/`

### Backend
- **Framework**: Tauri
- **Language**: Rust
- **Audio Processing**: Custom logic using the `cpal` library
- **Directory**: `src-tauri/`

## Installation

### Prerequisites
- Node.js (v16 or higher)
- Rust (latest stable version)
- Tauri CLI: Install using `cargo install tauri-cli`

### Steps
1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd voice-input-app
   ```
2. Install dependencies:
   ```bash
   npm install
   ```
3. Start the development server:
   ```bash
   npm run tauri dev
   ```

## Build

To build the application for production:
```bash
npm run tauri build
```

## Directory Structure

```
voice-input-app/
├── src/                # Frontend code
│   ├── components/     # React components
│   ├── hooks/          # Custom React hooks
│   ├── stores/         # Zustand stores
│   ├── utils/          # Utility functions
├── src-tauri/          # Backend code
│   ├── audio/          # Audio processing logic
│   ├── commands/       # Tauri commands
│   ├── recognition/    # Speech recognition logic
│   ├── utils/          # Utility modules
├── package.json        # Node.js dependencies
├── Cargo.toml          # Rust dependencies
├── vite.config.js      # Vite configuration
├── tailwind.config.js  # Tailwind CSS configuration
```

## Development

### Frontend
- Edit React components in `src/components/`.
- Use Zustand for state management in `src/stores/`.
- Add custom hooks in `src/hooks/`.

### Backend
- Add new Tauri commands in `src-tauri/src/commands/`.
- Implement audio processing logic in `src-tauri/src/audio/`.
- Extend speech recognition in `src-tauri/src/recognition/`.

## Contributing

Contributions are welcome! Please follow these steps:
1. Fork the repository.
2. Create a new branch for your feature or bugfix.
3. Commit your changes and push the branch.
4. Open a pull request.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Tauri](https://tauri.app/) for the backend framework.
- [React](https://reactjs.org/) for the frontend framework.
- [Zustand](https://zustand-demo.pmnd.rs/) for state management.
- [Tailwind CSS](https://tailwindcss.com/) for styling.
- [cpal](https://github.com/RustAudio/cpal) for audio processing in Rust.
