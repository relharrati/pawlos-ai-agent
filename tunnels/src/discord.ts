import { Client, GatewayIntentBits, Message, Partials } from 'discord.js';
import { chat } from './pawlos-client';

const sessions = new Map<string, string>();

export function startDiscord(token: string, allowedChannels: string[]): void {
  const client = new Client({
    intents: [
      GatewayIntentBits.Guilds,
      GatewayIntentBits.GuildMessages,
      GatewayIntentBits.MessageContent,
      GatewayIntentBits.DirectMessages,
    ],
    partials: [Partials.Channel],
  });

  client.once('ready', () => {
    console.log(`🤖 Discord tunnel ready as ${client.user?.tag}`);
  });

  client.on('messageCreate', async (msg: Message) => {
    if (msg.author.bot) return;

    // Allow DMs or whitelisted channels
    const isDM = !msg.guild;
    const channelName = isDM ? 'dm' : (msg.channel as { name?: string }).name ?? '';
    if (!isDM && allowedChannels.length > 0 && !allowedChannels.includes(channelName)) {
      return;
    }

    // Strip bot mention prefix
    let text = msg.content;
    if (client.user && text.startsWith(`<@${client.user.id}>`)) {
      text = text.slice(`<@${client.user.id}>`.length).trim();
    }
    if (!text) return;

    const sessionKey = `discord-${msg.author.id}`;
    const sessionId = sessions.get(sessionKey);

    try {
      // Send typing indicator if supported
      if ('sendTyping' in msg.channel) {
        await (msg.channel as any).sendTyping();
      }
      const response = await chat(text, sessionId);
      sessions.set(sessionKey, response.session_id);

      // Discord message limit is 2000 chars
      const chunks = splitMessage(response.content, 2000);
      for (const chunk of chunks) {
        await msg.reply(chunk);
      }
    } catch (e) {
      console.error('Discord error:', e);
      await msg.reply('⚠️ Error communicating with pawlos.');
    }
  });

  client.login(token);
}

function splitMessage(text: string, limit: number): string[] {
  const chunks: string[] = [];
  while (text.length > limit) {
    chunks.push(text.slice(0, limit));
    text = text.slice(limit);
  }
  if (text) chunks.push(text);
  return chunks;
}
