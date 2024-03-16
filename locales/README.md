# i18n ReadME

This folder contains internationalization (i18n) files for translating the bot messages into different languages. When translating, follow these instructions:

1. **File Naming:** Create a new file named `<language_code>.yml` for your language within this folder.
   
2. **Base File:** Start by copying the contents of the `en.yml` file into your newly created file.

3. **Translation:** Replace the English strings with their translated counterparts in your language. Ensure that the structure of the file remains intact, including placeholders and formatting.

4. **Testing:** Test the translations thoroughly to ensure accuracy and readability.
  - I'd recommend [i18n Ally](https://marketplace.visualstudio.com/items?itemName=Lokalise.i18n-ally) for VSCode to help you with the translations.

5. **Contributing:** If you've made significant improvements or translations for a language that doesn't exist yet, consider submitting a pull request to the repository.

## Example Translation File Structure

```yaml
_version: 1
referenced: "🔗 Your message was referenced by <@%{USER_ID}< (%{AUTHOR_NICKNAME}) in: %{MESSAGE_URL}"
error_sending_message: "🚨 Error sending message: %{WHY}"
nothing_selected: "🚨 Nothing selected"
```

## Notes

- **Placeholder Usage:** Do not alter the placeholders (`%{...}`). They are used for dynamic content insertion and should be retained in your translations.
  
- **URLs and Links:** Be cautious when translating URLs or links. Ensure that they remain functional and relevant in your language context.

- **Consistency:** Maintain consistency in tone, style, and formatting throughout the translation for a seamless user experience.

- **Fallback Language:** If a user's language is not supported or the specific string hasn't been translated, the bot will default to English.

Thank you for your contribution to making the bot accessible to users worldwide! 🌍✨
