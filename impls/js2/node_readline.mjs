import prompt from "readline-sync";

export function readline() {
  return prompt.prompt({ prompt: "user> " });
}
