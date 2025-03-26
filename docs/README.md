![TacoQ Banner](https://raw.githubusercontent.com/taco-xyz/tacoq/7f5a946229a3bdd63e94d7306720d0bccadd2f6e/TacoQBanner.png)

# TacoQ Documentation

![Git Tag](https://img.shields.io/github/v/tag/taco-xyz/tacoq)
![CI](https://img.shields.io/github/actions/workflow/status/taco-xyz/tacoq/.github%2Fworkflows%2Ftest.yml)
![Github Stars](https://img.shields.io/github/stars/taco-xyz/tacoq)

This is the official documentation website for TacoQ, a multi-language distributed task queue with built-in observability, low latency, and first-class idiomatic support.

## About TacoQ

TacoQ is a task queue system that allows you to schedule tasks to be executed asynchronously in workers outside your application. Key features include:

- Multi-language interoperability
- Built-in observability
- Low latency
- First-class idiomatic support
- REST API integration

## Development

This documentation site is built with [Next.js](https://nextjs.org) and uses MDX for content management.

To run the development server:

```bash
# Install dependencies
yarn install

# Start development server
yarn dev
```

Open [http://localhost:3000](http://localhost:3000) to view the documentation.

## Documentation Structure

- **Quickstart**
  - Core Concepts
  - Setup Guide
- **Technical Reference**
  - System Architecture
  - Versioning
  - Relay Configuration
  - Benchmarks
- **Guide**
  - Horizontal-Scaling
  - Hot-Reloading
  - Maximizing-Performance
  - Same-App-Worker-Pattern
  - Serialization
  - Task-Versioning

## Contributing

The documentation uses:

- MDX for content
- Tailwind CSS for styling
- Next.js App Router
- Custom components for interactive examples

## Deployment

This documentation is deployed on Vercel. Any changes to the main branch will automatically trigger a new deployment.

## Local Development

1. Clone the repository
2. Install dependencies: `yarn install`
3. Start development server: `yarn dev`
4. Visit `http://localhost:3000`

## License

This project is licensed under the MIT License.
For more details, please refer to the [MIT License](https://opensource.org/licenses/MIT).
