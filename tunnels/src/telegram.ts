import TelegramBot from 'node-telegram-bot-api';
import { chat } from './pawlos-client';

const sessions = new Map<number, string>();

export function startTelegram(token: string, allowedUsers: string[]): void {
  const bot = new TelegramBot(token, { polling: true });

  console.log('🤖 Telegram tunnel started');

  bot.on('message', async (msg) => {
    const chatId = msg.chat.id;
    const username = msg.from?.username ?? '';

    // Filter by allowed users if specified
    if (
      allowedUsers.length > 0 &&
      !allowedUsers.includes(`@${username}`) &&
      !allowedUsers.includes(username)
    ) {
      return;
    }

    const text = msg.text?.replace(/^\/\w+\s*/, ''); // strip commands like /start
    if (!text) return;

    const sessionId = sessions.get(chatId);

    try {
      await bot.sendChatAction(chatId, 'typing');
      const response = await chat(text, sessionId);
      sessions.set(chatId, response.session_id);

      // Send with Markdown parse mode
      await bot.sendMessage(chatId, response.content, {
        parse_mode: 'Markdown',
        reply_to_message_id: msg.message_id,
      });
    } catch (e) {
      console.error('Telegram error:', e);
      await bot.sendMessage(chatId, '⚠️ Error communicating with pawlos.');
    }
  });

  bot.onText(/\/start/, (msg) => {
    bot.sendMessage(
      msg.chat.id,
      '🤖 *pawlos is awake.* Send me a message!',
      { parse_mode: 'Markdown' },
    );
  });
}
