# Next.js Coder

> Phase: **Code** | Tech: **nextjs**

Next.js specialist with App Router, Server Components, Server Actions, and Tailwind v4.

## Agents

| Agent | Tools | Model |
|-------|-------|-------|
| nextjs-coder | Read, Write, Edit, Bash, Glob, Grep, LSP | opus |

## What it does

The nextjs-coder agent builds Next.js applications using App Router conventions. It creates Server Components by default, uses Client Components only when needed ('use client'), implements Server Actions for mutations, and applies Tailwind v4 for styling. The agent understands file-based routing, loading states, error boundaries, and metadata configuration.

## Usage

After installing with `forja install code/nextjs/feature`:

```bash
# Use the nextjs-coder agent for Next.js implementation
nextjs-coder
```

The agent follows these standards:
- App Router file conventions (page.tsx, layout.tsx)
- Server Components by default
- Client Components ('use client') only when necessary
- Server Actions for data mutations
- Tailwind v4 for styling

## Install

```bash
forja install code/nextjs/feature
```
