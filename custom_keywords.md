# Custom Keywords

Words added to Handy's speech-to-text custom words list to improve transcription accuracy for programming vocabulary.

## Current Keywords

```
PostgreSQL  Supabase    FastAPI     Logfire     TypeScript
JavaScript  Celery      Zustand     Tailwind    Vite
ESLint      Vercel      useState    useEffect   WebSocket
OAuth       middleware  webhook     localhost   README
Dockerfile  async       pnpm        refactor    Drizzle
```

## For Future Agents

Custom words are stored in:

```
~/Library/Application Support/com.pais.handy/settings_store.json
```

Edit the `custom_words` array in that file:

```json
"custom_words": ["PostgreSQL", "Supabase", ...]
```

**Constraints:**
- Single words only (no spaces)
- Max 50 characters per word
- No special characters (`<>"'&`)

Restart Handy after editing the file for changes to take effect.
