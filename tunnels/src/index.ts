import { startDiscord } from './discord';
import { startTelegram } from './telegram';

const DISCORD_TOKEN         = process.env.DISCORD_TOKEN ?? '';
const DISCORD_CHANNELS      = (process.env.DISCORD_ALLOWED_CHANNELS ?? '').split(',').filter(Boolean);
const TELEGRAM_TOKEN        = process.env.TELEGRAM_TOKEN ?? '';
const TELEGRAM_ALLOWED      = (process.env.TELEGRAM_ALLOWED_USERS ?? '').split(',').filter(Boolean);

if (DISCORD_TOKEN) {
  startDiscord(DISCORD_TOKEN, DISCORD_CHANNELS);
} else {
  console.log('DISCORD_TOKEN not set — Discord tunnel disabled.');
}

if (TELEGRAM_TOKEN) {
  startTelegram(TELEGRAM_TOKEN, TELEGRAM_ALLOWED);
} else {
  console.log('TELEGRAM_TOKEN not set — Telegram tunnel disabled.');
}

if (!DISCORD_TOKEN && !TELEGRAM_TOKEN) {
  console.error('No tunnel tokens configured. Set DISCORD_TOKEN or TELEGRAM_TOKEN.');
  process.exit(1);
}
