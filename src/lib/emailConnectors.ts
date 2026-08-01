export type EmailConnectorId =
  | "icloud"
  | "gmail"
  | "outlook"
  | "yahoo"
  | "fastmail"
  | "custom";

export type EmailConnector = {
  id: EmailConnectorId;
  name: string;
  description: string;
  host: string;
  port: number;
  mailbox: string;
  /** Shown in the connect flow — app-password / OAuth hints. */
  setupHint: string;
  helpUrl?: string;
};

export const EMAIL_CONNECTORS: EmailConnector[] = [
  {
    id: "icloud",
    name: "iCloud Mail",
    description: "Apple ID with an app-specific password",
    host: "imap.mail.me.com",
    port: 993,
    mailbox: "INBOX",
    setupHint:
      "Create an app-specific password at appleid.apple.com, then paste it below.",
    helpUrl: "https://support.apple.com/en-us/HT204397",
  },
  {
    id: "gmail",
    name: "Gmail",
    description: "Google account with an app password",
    host: "imap.gmail.com",
    port: 993,
    mailbox: "INBOX",
    setupHint:
      "Enable IMAP in Gmail settings and use a Google app password (2FA required).",
    helpUrl: "https://support.google.com/mail/answer/7126229",
  },
  {
    id: "outlook",
    name: "Outlook / Microsoft 365",
    description: "Outlook.com or work/school Microsoft mail",
    host: "outlook.office365.com",
    port: 993,
    mailbox: "INBOX",
    setupHint:
      "Use your full email as the username. If MFA is on, create an app password in your Microsoft account security settings.",
    helpUrl: "https://support.microsoft.com/en-us/office/pop-imap-and-smtp-settings",
  },
  {
    id: "yahoo",
    name: "Yahoo Mail",
    description: "Yahoo with an app password",
    host: "imap.mail.yahoo.com",
    port: 993,
    mailbox: "INBOX",
    setupHint: "Generate an app password in Yahoo Account Security, then connect.",
    helpUrl: "https://help.yahoo.com/kb/generate-manage-third-party-passwords-sln15241.html",
  },
  {
    id: "fastmail",
    name: "Fastmail",
    description: "Fastmail IMAP",
    host: "imap.fastmail.com",
    port: 993,
    mailbox: "INBOX",
    setupHint: "Use an app password from Fastmail Settings → Privacy & Security.",
    helpUrl: "https://www.fastmail.com/help/technical/ssltlsstarttls.html",
  },
  {
    id: "custom",
    name: "Other IMAP",
    description: "Enter host and credentials manually",
    host: "",
    port: 993,
    mailbox: "INBOX",
    setupHint: "Ask your provider for IMAP host, port (usually 993), and an app password.",
  },
];

export function getEmailConnector(id: EmailConnectorId): EmailConnector {
  return (
    EMAIL_CONNECTORS.find((c) => c.id === id) ?? EMAIL_CONNECTORS[EMAIL_CONNECTORS.length - 1]!
  );
}
