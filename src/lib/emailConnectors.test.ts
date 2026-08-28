import { connectorIdForHost, emailAuthLabel } from "./emailConnectors.ts";

function assertEqual(actual: string, expected: string) {
  if (actual !== expected) {
    throw new Error(`expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
  }
}

assertEqual(emailAuthLabel({ auth: "oauth", provider: "google" }), "Google (browser)");
assertEqual(
  emailAuthLabel({ auth: "oauth", provider: "microsoft" }),
  "Microsoft (browser)",
);
assertEqual(emailAuthLabel({ auth: "mailapp", provider: "mailapp" }), "Mail.app");
assertEqual(emailAuthLabel({ auth: "password", provider: "imap" }), "IMAP");
assertEqual(connectorIdForHost("imap.mail.me.com"), "icloud");
assertEqual(connectorIdForHost("imap.fastmail.com"), "fastmail");
assertEqual(connectorIdForHost("imap.gmail.com"), "custom");

console.log("emailConnectors tests ok");
