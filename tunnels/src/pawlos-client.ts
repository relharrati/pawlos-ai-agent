import axios from 'axios';

const PAWLOS_URL = process.env.PAWLOS_URL ?? 'http://localhost:9797';

interface ChatResponse {
  session_id: string;
  content: string;
}

/** Send a message to the pawlos core and return the response */
export async function chat(
  message: string,
  sessionId?: string,
): Promise<ChatResponse> {
  const body: Record<string, string> = { message };
  if (sessionId) body['session_id'] = sessionId;

  const res = await axios.post<ChatResponse>(`${PAWLOS_URL}/api/chat`, body);
  return res.data;
}
