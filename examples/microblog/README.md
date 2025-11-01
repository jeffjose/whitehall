# Microblog - Complete Example App

A Twitter/Threads-style microblogging application built with Whitehall.

## Purpose

This app serves as:
1. **Syntax validation** - Tests if Whitehall syntax works well in practice
2. **Reference implementation** - Shows best practices and patterns
3. **Decision driver** - Surfaces pending syntax decisions as we build

## Features

- ✅ **Authentication** - Login and signup flow
- ✅ **Home Feed** - Timeline of posts from followed users
- ✅ **User Profiles** - View user info and their posts
- ✅ **Create Posts** - Compose new posts with validation
- ✅ **Post Details** - Single post view with comments
- ✅ **Settings** - User preferences and account management

## Structure

```
microblog/
├── whitehall.toml              # Project configuration
├── src/
│   ├── routes/                 # File-based routing
│   │   ├── +page.wh           # Home/Feed
│   │   ├── login/
│   │   │   └── +page.wh       # Login screen
│   │   ├── signup/
│   │   │   └── +page.wh       # Signup screen
│   │   ├── profile/
│   │   │   └── [id]/
│   │   │       └── +page.wh   # User profile
│   │   ├── post/
│   │   │   ├── create/
│   │   │   │   └── +page.wh   # Create post
│   │   │   └── [id]/
│   │   │       └── +page.wh   # Post detail
│   │   └── settings/
│   │       └── +page.wh       # Settings
│   ├── components/            # Reusable components
│   │   ├── PostCard.wh
│   │   ├── UserAvatar.wh
│   │   ├── CommentItem.wh
│   │   └── ...
│   ├── models/                # Data models
│   │   ├── User.kt
│   │   ├── Post.kt
│   │   └── Comment.kt
│   └── lib/                   # Utilities
│       └── api/
│           └── ApiClient.kt
└── README.md
```

## Syntax Patterns Demonstrated

### Component Structure
- Props with `@prop val`
- State with `var`/`val`
- Functions and computed values
- Lifecycle hooks (`onMount`, `onUnmount`)

### UI Patterns
- Control flow (`@if`, `@for`, `@when`)
- Data binding (`bind:value`, `bind:checked`)
- Event handling
- Conditional rendering
- List rendering with keys and empty states

### Navigation
- File-based routing
- Type-safe navigation with @Serializable routes
- Route parameters
- Deep linking

### Data Flow
- Component state
- State lifting
- Props drilling
- Shared state patterns

### Forms
- Input validation
- Error handling
- Submit handling
- Form state management

## Running

```bash
# Once compiler is built
cd examples/microblog
whitehall run
```

## Decisions Made While Building

As we build this app, pending syntax decisions are finalized and documented:
- See `docs/syntax/decisions/` for finalized decisions
- See `docs/syntax/PENDING.md` for remaining open questions
