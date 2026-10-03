export const SAMPLE_SOURCE = `{
  "nav": {
    "home": "Home",
    "settings": "Settings",
    "signOut": "Sign out"
  },
  "checkout": {
    "title": "Checkout",
    "items": "{count, plural, one {# item} other {# items}}",
    "total": "Total: {total, number, ::currency/EUR}"
  },
  "errors": {
    "offline": "You are offline. Changes will sync when you reconnect."
  }
}
`;

export const SAMPLE_TARGET = `{
  "nav": {
    "home": "Accueil"
  }
}
`;
