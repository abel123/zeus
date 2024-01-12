import { createHandler } from '@mui/toolpad';
import express from 'express';

const app = express();

// Initialize the Toolpad handler. Make sure to pass the base path
const { handler } = await createHandler({
  dev: process.env.NODE_ENV === 'development',
  base: '/app',
});

// Use the handler in your application
app.use('/app', handler);

app.listen(3001);