// =============================================================================
//  i18n — دوزبانه: English + فارسی
//  انتخاب کاربر در localStorage می‌ماند و عمداً جدا از profile است تا با
//  «بازنشانی به تنظیمات پیش‌فرض» زبان کاربر عوض نشود.
//
//  قاعده: کلیدها همان جمله‌های انگلیسی رابط هستند؛ اگر ترجمه‌ای نبود
//  همان انگلیسی نمایش داده می‌شود. واژه‌های لاتین داخل جمله‌های فارسی در
//  <bdi> می‌نشینند تا چپ‌به‌راست رندر شوند و متن راست‌به‌چپ بهم نریزد
//  (فقط در رشته‌هایی که با innerHTML رندر می‌شوند).
// =============================================================================

const STORAGE_KEY = 'aether.lang'

export const LANGS = [
  ['en', 'English'],
  ['fa', 'فارسی'],
]

const FA = {
  // --- v12 (۱.۲.۴-p4): صفحهٔ گفت‌وگو، پورت از AiChatScreen.kt ---
  // ترجمه‌ها همان `ai_chat_*` در values-fa/strings.xml هستند؛ جملهٔ تازه‌ای
  // ساخته نشده تا کاربری که موبایل را می‌شناسد همان کلمات را ببیند.
  'Chat': 'گفت‌وگو',
  'Chat with Gemini': 'گفت‌وگو با جمینای',
  'Open the chat': 'باز کردن گفت‌وگو',
  'Ask anything, or say what you want changed': 'هر چیزی بپرسید، یا بگویید چه چیزی عوض شود',
  'Model: {0}': 'مدل: {0}',
  '{0} message(s) in the conversation': '{0} پیام در گفت‌وگو',
  'Hi, I am Aether AI': 'سلام، من هوش مصنوعی اتر هستم',
  'I can explain any setting in this app, read this session’s log and propose tuning, or just answer a question.':
    'می‌توانم هر تنظیمی در این برنامه را توضیح بدهم، لاگ همین نشست را بخوانم و تنظیمات پیشنهاد کنم، یا فقط به سؤالتان جواب بدهم.',
  'Why is my connection slow right now?': 'چرا اتصالم الان کند است؟',
  'Which protocol should I use on mobile data?': 'روی دیتای همراه چه پروتکلی بهتر است؟',
  'Explain MTU and pick the best one for me': 'MTU را توضیح بده و بهترینش را انتخاب کن',
  'Make the tunnel harder to detect': 'تونل را سخت‌تر قابل تشخیص کن',
  'Ask Aether AI…': 'از هوش مصنوعی اتر بپرسید…',
  'Stop': 'توقف',
  'Clear the conversation': 'پاک کردن گفت‌وگو',
  'Thinking…': 'در حال فکر کردن…',
  'Copy answer': 'کپی پاسخ',
  'Copied': 'کپی شد',
  'Could not copy — your system refused clipboard access.': 'کپی نشد — سیستم شما اجازهٔ دسترسی به کلیپ‌بورد را نداد.',
  'Try again': 'تلاش مجدد',
  'Edit': 'ویرایش',
  'Delete': 'حذف',
  'edited': 'ویرایش‌شده',
  'Cancel': 'انصراف',
  'Edit message': 'ویرایش پیام',
  'Send again': 'ارسال دوباره',
  'Everything after this message will be removed and the assistant will answer the edited question.':
    'هر چیزی پس از این پیام حذف می‌شود و دستیار به پرسش ویرایش‌شده پاسخ می‌دهد.',
  '{0} selected': '{0} مورد انتخاب شد',
  'Select all': 'انتخاب همه',
  'Cancel selection': 'لغو انتخاب',
  'Delete selected': 'حذف موارد انتخاب‌شده',
  // تأییدِ حذف — یک دیالوگ برای هر سه مسیر (سطلِ حباب، حذفِ گروهی، پاک‌کردنِ همه).
  // متن‌ها از `ai_chat_delete_*` در `values-fa/strings.xml` موبایل می‌آیند.
  'Delete {0} message(s)?': '{0} پیام حذف شود؟',
  'This cannot be undone. Deleted messages are no longer sent to the assistant as context.':
    'این کار قابل بازگشت نیست. پیام‌های حذف‌شده دیگر به‌عنوان زمینهٔ گفت‌وگو برای دستیار فرستاده نمی‌شوند.',
  'Clear the whole conversation?': 'کلِ گفت‌وگو پاک شود؟',
  Clear: 'پاک کردن',
  // کارتِ تنظیماتِ پیشنهادی و دیالوگِ پس از اعمال — از `ai_changes_title`،
  // `ai_apply`، `ai_applied`، `ai_apply_note` و `ai_applied_dialog_*` موبایل.
  'Proposed settings ({0})': 'تنظیمات پیشنهادی ({0})',
  Apply: 'اعمال',
  Applied: 'اعمال شد',
  'Tunnel settings are handed to the engine when it starts, so these take effect on your next connect.':
    'تنظیمات تونل هنگام راه‌اندازی به موتور داده می‌شوند، پس این تغییرها در اتصال بعدی اثر می‌گذارند.',
  'Saved for the next connection': 'برای اتصال بعدی ذخیره شد',
  'The new settings are stored, but the tunnel is already running with the old ones. Tunnel settings are handed to the engine when it starts, so disconnect and connect again for them to take effect.':
    'تنظیمات جدید ذخیره شد، اما تونل همچنان با تنظیمات قبلی در حال اجراست. تنظیمات تونل هنگام شروع به موتور داده می‌شود؛ پس یک‌بار قطع و دوباره وصل کنید تا اعمال شوند.',
  'Got it': 'متوجه شدم',
  // پایکِ حبابِ ✨ و پرسشی که به گفت‌وگو منتقل می‌کند.
  'Did not understand? Ask the assistant': 'متوجه نشدید؟ از دستیار بپرسید',
  'Explain this more simply: “{0}”. This is what the app told me: {1}':
    'این را ساده‌تر توضیح بده: «{0}». چیزی که برنامه به من گفت این بود: {1}',
  // جمله‌های شکست — از `errorKind` ساخته می‌شوند، نه از متنِ خامِ گوگل.
  'Google rejected this key. Check it in AI Studio, or paste it again.':
    'گوگل این کلید را رد کرد. آن را در <bdi>AI Studio</bdi> بررسی کنید یا دوباره بچسبانید.',
  'The free quota for this key is used up for now. Try again later.':
    'سهمیهٔ رایگان این کلید فعلاً تمام شده. بعداً دوباره تلاش کنید.',
  'This key cannot use the selected model. Discover the models again.':
    'این کلید نمی‌تواند از مدل انتخاب‌شده استفاده کند. مدل‌ها را دوباره کشف کنید.',
  'The request never reached Google. Check the tunnel and try again.':
    'درخواست هرگز به گوگل نرسید. تونل را بررسی کنید و دوباره تلاش کنید.',
  'Google failed on its own side. This is not your connection — try again.':
    'گوگل سمت خودش شکست خورد. مشکل از اتصال شما نیست — دوباره تلاش کنید.',
  'Google sent back something this app could not read.': 'گوگل چیزی برگرداند که این برنامه نتوانست بخواند.',
  'The answer was cut off before it finished.': 'پاسخ پیش از تمام‌شدن بریده شد.',
  'The model returned no answer.': 'مدل هیچ پاسخی برنگرداند.',
  'The request failed.': 'درخواست شکست خورد.',
  // --- v12 (۱.۲.۳): بک‌اند ترابرد زنجیره‌ای ---
  'Transport': 'ترابرد',
  'Backend': 'بک‌اند',
  'Exit country': 'کشور خروج',
  'Automatic': 'خودکار',
  'Aether alone exits through a Cloudflare WARP edge. Chaining Psiphon keeps Aether as the first hop and swaps the exit for an ordinary hosting IP, which is what opens sites that reject WARP ranges.':
    'اِتِر تنها از یک لبهٔ <bdi>Cloudflare WARP</bdi> بیرون می‌رود. زنجیره‌کردن <bdi>Psiphon</bdi> هاپ اول را همان اِتِر نگه می‌دارد و خروجی را با یک آی‌پی هاستینگ عادی عوض می‌کند؛ همین است که سایت‌هایی را باز می‌کند که رنج‌های <bdi>WARP</bdi> را رد می‌کنند.',
  'Only applies to the chained backend. If no server is reachable in that country, Aether falls back to an automatic exit instead of hanging.':
    'فقط برای بک‌اند زنجیره‌ای است. اگر در آن کشور هیچ سروری در دسترس نباشد، اِتِر به‌جای معلق‌ماندن به خروجی خودکار برمی‌گردد.',
  'Starting the Psiphon stage…': 'در حال راه‌اندازی مرحلهٔ <bdi>Psiphon</bdi>…',
  'Rebuilding the chain…': 'بازسازی زنجیره…',
  // --- v12 (۱.۲.۴): دستیار هوش مصنوعی ---
  'Assistant': 'دستیار',
  'Gemini API key': 'کلید API جمینای',
  'The key is stored sealed on this PC with Windows DPAPI and is never written to the log.':
    'کلید روی همین رایانه با <bdi>DPAPI</bdi> ویندوز مهر و ذخیره می‌شود و هرگز در لاگ نوشته نمی‌شود.',
  'Save': 'ذخیره',
  'Forget': 'فراموش کن',
  'No key stored.': 'کلیدی ذخیره نشده.',
  'A key ending in …{0} is stored.': 'کلیدی که به …{0} ختم می‌شود ذخیره است.',
  'Get a free key from Google AI Studio': 'یک کلید رایگان از <bdi>Google AI Studio</bdi> بگیرید',
  'Model': 'مدل',
  'Only fast Flash-class models are offered: they answer on a free key.':
    'فقط مدل‌های سریعِ ردهٔ <bdi>Flash</bdi> پیشنهاد می‌شوند؛ همان‌هایی که روی کلید رایگان پاسخ می‌دهند.',
  'Discover models for this key': 'کشف مدل‌های این کلید',
  'No models discovered yet.': 'هنوز مدلی کشف نشده.',
  'Add a key first.': 'اول یک کلید وارد کنید.',
  'Tune for my network': 'تنظیم برای شبکهٔ من',
  'Reads the recent connection log — with addresses masked and identifiers removed — and changes at most one or two transport settings.':
    'لاگ اخیر اتصال را می‌خواند — با نشانی‌های ماسک‌شده و شناسه‌های حذف‌شده — و حداکثر یکی دو تنظیم ترابرد را عوض می‌کند.',
  'Analyse and tune': 'تحلیل و تنظیم',
  'Changed': 'تغییر داده شد',
  'Nothing needed changing.': 'چیزی نیاز به تغییر نداشت.',
  'Refused by the app': 'ردشده توسط برنامه',
  'Dismiss': 'بستن',
  'Ask anything': 'هر چه می‌خواهید بپرسید',
  'Ask about a setting, an error, or censorship…': 'دربارهٔ یک تنظیم، یک خطا، یا فیلترینگ بپرسید…',
  'Send': 'بفرست',
  'Clear conversation': 'پاک‌کردن گفت‌وگو',
  'Close': 'بستن',
  'Ask the assistant about this setting': 'از دستیار دربارهٔ این تنظیم بپرس',
  'Add your Gemini API key to use the assistant.': 'برای استفاده از دستیار، کلید <bdi>API</bdi> جمینای را وارد کنید.',
  'No model has been discovered for this key yet.': 'هنوز هیچ مدلی برای این کلید کشف نشده است.',
  'The assistant needs the tunnel to be connected — Google is not reachable otherwise.':
    'دستیار به تونلِ وصل نیاز دارد — وگرنه گوگل در دسترس نیست.',
  'Switch the connection to Aether → Psiphon: Google refuses the Cloudflare WARP addresses that Aether alone exits from.':
    'اتصال را به «اِتِر ← <bdi>Psiphon</bdi>» عوض کنید: گوگل نشانی‌های <bdi>Cloudflare WARP</bdi> را که اِتِرِ تنها از آن‌ها بیرون می‌رود رد می‌کند.',
  'The assistant is not available right now.': 'دستیار در این لحظه در دسترس نیست.',
  // --- v12 (۱.۲.۴): منوی تنظیمات (پورت a2) ---
  'Settings': 'تنظیمات',
  'Tunnel settings': 'تنظیمات تونل',
  'Back': 'بازگشت',
  'On': 'روشن',
  'Tunnel': 'تونل',
  'Connection': 'اتصال',
  'Network backend, exit country, protocol and scanning': 'بک‌اند شبکه، کشور خروج، پروتکل و اسکن',
  'Transport & anti-DPI': 'ترابرد و ضدDPI',
  'Obfuscation, endpoint, MTU and anti-DPI': 'مبهم‌سازی، اندپوینت، <bdi>MTU</bdi> و ضدDPI',
  'DNS & routing rules': 'DNS و قواعد مسیریابی',
  'Resolvers inside the tunnel, block and bypass lists': 'ریزالورهای داخل تونل، فهرست‌های مسدود و عبور',
  'Upstream proxy (chaining)': 'پروکسی بالادست (زنجیره‌ای)',
  'Dial out through a proxy already running on this PC': 'خروج از طریق پروکسی‌ای که همین حالا روی این رایانه اجراست',
  'Kill switch & leak protection': 'قطع‌کن و محافظت از نشتی',
  'What happens the moment the tunnel drops': 'وقتی تونل می‌افتد چه می‌شود',
  'Join a Cloudflare organisation instead of plain WARP': 'به‌جای <bdi>WARP</bdi> ساده، عضو یک سازمان <bdi>Cloudflare</bdi> شوید',
  'Application': 'برنامه',
  'Apps & LAN sharing': 'برنامه‌ها و اشتراک شبکه',
  'Which programs use the tunnel, and who else may': 'کدام برنامه‌ها از تونل استفاده کنند، و چه کسی دیگر اجازه دارد',
  'Interface language': 'زبان رابط کاربری',
  'Reset all settings to defaults': 'بازنشانی همهٔ تنظیمات به پیش‌فرض',
  'Every setting goes back to its default, including endpoint ranges, routing rules and enrolment details.':
    'هر تنظیم به مقدار پیش‌فرضش برمی‌گردد، از جمله بازه‌های اندپوینت، قواعد مسیریابی و جزئیات ثبت‌نام.',
  'Reset every setting to its default?': 'همهٔ تنظیمات به پیش‌فرض برگردند؟',
  'The tunnel is running. Changes are saved now and handed to the engine the next time it starts — reconnect to apply them.':
    'تونل در حال اجراست. تغییرها همین حالا ذخیره می‌شوند و در استارت بعدی به هسته داده می‌شوند — برای اعمال، دوباره وصل شوید.',
  // --- پوسته (منو + نوار عنوان) ---
  'Home': 'خانه',
  'Advanced': 'پیشرفته',
  'Diagnostics': 'عیب‌یابی',
  'Share over LAN': 'اشتراک در شبکه',
  'About': 'درباره',
  'Menu': 'منو',
  'Aether': 'اِتِر',

  // --- صفحهٔ اصلی ---
  'Freedom, in one tap': 'آزادی، با یک لمس',
  'Tap to connect securely': 'برای اتصال امن لمس کنید',
  'Tap to disconnect': 'برای قطع اتصال لمس کنید',
  'Something went wrong': 'مشکلی پیش آمد',
  'Verifying connection…': 'در حال راستی‌آزمایی اتصال…',
  'Disconnected': 'قطع',
  'Starting engine…': 'در حال راه‌اندازی موتور…',
  'Connecting…': 'در حال اتصال…',
  'Verifying…': 'در حال راستی‌آزمایی…',
  'Connected': 'متصل',
  'Reconnecting…': 'اتصال دوباره…',
  'Disconnecting…': 'در حال قطع اتصال…',
  'Connection failed': 'اتصال ناموفق بود',
  'Your IP': 'آی‌پی شما',
  'Server IP': 'آی‌پی سرور',
  'Checking IP…': 'در حال بررسی آی‌پی…',
  'IP unavailable': 'آی‌پی در دسترس نیست',
  'Connected for': 'مدت اتصال',
  'Protocol': 'پروتکل',
  'Endpoint': 'نقطهٔ اتصال',
  'Latency': 'تأخیر',
  'Download': 'دانلود',
  'Upload': 'آپلود',

  // --- ۱.۲.۴: کارت اتصال (پورت از ConnectionCard.kt) ---
  'Total': 'مجموع',
  'Ping strength': 'قدرت پینگ',
  'Excellent': 'عالی',
  'Good': 'خوب',
  'Fair': 'متوسط',
  'Poor': 'ضعیف',
  'Measuring…': 'در حال سنجش…',
  'Offline': 'قطع',

  // --- v1.2.0: محافظت در برابر نشتی WebRTC ---
  'Connection safety': 'امنیت اتصال',
  'Kill switch': 'کیل‌سوییچ',
  'Block browser traffic if the tunnel drops': 'اگر تونل قطع شد، ترافیک مرورگرها را مسدود می‌کند',
  'IPv6 leak protection': 'محافظت در برابر نشت IPv6',
  'Keep the IPv6 default route protected or block it safely': 'مسیر پیش‌فرض IPv6 را داخل مسیر امن نگه می‌دارد یا ایمن مسدود می‌کند',
  'Automatic reconnect attempts': 'تعداد تلاش‌های اتصال مجدد خودکار',
  'WebRTC protected — no IP leak': 'WebRTC محافظت شد — بدون نشت آی‌پی',
  'WebRTC is leaking your real IP': 'WebRTC آی‌پی واقعی شما را لو می‌دهد',
  'Checking for WebRTC leaks…': 'در حال بررسی نشتی WebRTC…',
  'WebRTC leak test': 'آزمایش نشتی WebRTC',
  'Testing for WebRTC leaks…': 'در حال آزمایش نشتی WebRTC…',

  // --- تنظیمات پیشرفته ---
  'Language': 'زبان برنامه',
  'Scan mode': 'حالت اسکن',
  'IP version': 'نسخهٔ آی‌پی',
  'Noize': 'نویز',
  // v17: گزینه‌های حالت اسکن و نویز — تا کل صفحهٔ پیشرفته فارسی باشد.
  'Turbo': 'توربو',
  'Balanced': 'متعادل',
  'Thorough': 'موشکافانه',
  'Stealth': 'پنهان‌کار',
  'Ironclad': 'آهنین',
  'Light': 'ملایم',
  'Firewall': 'فایروال',
  'GFW': 'فیلترینگ چین (GFW)',
  'Aggressive': 'تهاجمی',
  'Automatic': 'خودکار',
  'Manual peer': 'سرور دستی',
  'Manual range': 'بازهٔ دستی',
  'Peer address': 'آدرس سرور',
  'Address range': 'بازهٔ آدرس',
  'Off': 'خاموش',
  'Both': 'هر دو',
  'Quick reconnect': 'اتصال مجدد سریع',
  'Reconnect instantly after a drop': 'بعد از قطعی بلافاصله دوباره وصل می‌شود',
  'MASQUE over HTTP/2': '<bdi>MASQUE</bdi> روی <bdi>HTTP/2</bdi>',
  'Helps on networks that block HTTP/3': 'برای شبکه‌هایی که <bdi>HTTP/3</bdi> را مسدود می‌کنند',
  'Packet fragmentation': 'قطعه‌قطعه‌سازی بسته‌ها',
  'Splits the handshake to evade filtering': 'دست‌دادن <bdi>TLS</bdi> را تکه‌تکه می‌کند تا از فیلترینگ عبور کند',
  'Encrypted Client Hello (auto)': '<bdi>Encrypted Client Hello</bdi> (خودکار)',
  'Let other devices on your network use this tunnel': 'دستگاه‌های دیگر شبکه بتوانند از این تونل استفاده کنند',
  'Split tunneling': 'تونل تفکیکی',
  'Only these apps': 'فقط این برنامه‌ها',
  'All except these': 'همه به‌جز این‌ها',
  'Applications': 'برنامه‌ها',
  'One executable name per line.': 'در هر خط نام یک فایل اجرایی (<bdi>exe</bdi>).',

  // --- v10: Zero Trust، مسیریابی و DNS (هستهٔ 1.5.0) ---
  'Zero Trust': '<bdi>Zero Trust</bdi> (سازمانی)',
  'Team name': 'نام تیم (سازمان)',
  'Connect as a managed device of a Cloudflare Zero Trust organization. Leave empty for normal WARP.': 'اتصال به‌عنوان دستگاه مدیریت‌شدهٔ یک سازمان <bdi>Cloudflare Zero Trust</bdi>. برای <bdi>WARP</bdi> معمولی خالی بگذارید.',
  'Sign-in method': 'روش ورود',
  'Email code': 'کد ایمیلی',
  'Service token': 'توکن سرویس',
  'Access token': 'توکن دسترسی',
  'Access email': 'ایمیل ورود',
  'A one-time code is sent to this mailbox on connect.': 'هنگام اتصال، یک کد یک‌بارمصرف به این صندوق ایمیل فرستاده می‌شود.',
  'Stored in memory only — never written to disk.': 'فقط در حافظه نگه داشته می‌شود — هرگز روی دیسک نوشته نمی‌شود.',
  'Gateway proxy': 'پروکسی <bdi>Gateway</bdi>',
  "Route HTTP/HTTPS through your organization's Gateway (adds a hop and logs browsing)": 'عبور <bdi>HTTP/HTTPS</bdi> از <bdi>Gateway</bdi> سازمان (یک هاپ اضافه می‌کند و مرور شما را لاگ می‌کند)',
  'Routing rules': 'قوانین مسیریابی',
  'Blocked destinations': 'مقصدهای مسدود',
  'One rule per line — domain, IP or CIDR. These connections are refused.': 'در هر خط یک قاعده — دامنه، آی‌پی یا <bdi>CIDR</bdi>. این اتصال‌ها کاملاً رد می‌شوند.',
  'Direct destinations': 'مقصدهای مستقیم',
  'One rule per line. These bypass the tunnel — for banking apps, LAN services and domestic sites.': 'در هر خط یک قاعده. این مقصدها از تونل عبور نمی‌کنند — برای بانک، سرویس‌های شبکهٔ محلی و سایت‌های داخلی.',
  'In-tunnel DNS servers': 'سرورهای <bdi>DNS</bdi> داخل تونل',
  'Resolvers used inside the tunnel. Empty = engine defaults.': 'حل‌کننده‌های نام داخل تونل. خالی = پیش‌فرض موتور.',
  'These features need engine core 1.5.0 or newer. The bundled core is older, so they are disabled.': 'این قابلیت‌ها به هستهٔ <bdi>1.5.0</bdi> یا بالاتر نیاز دارند. هستهٔ همراه این بیلد قدیمی‌تر است، پس غیرفعال شده‌اند.',

  // --- v11: پروکسی بالادست، تشخیص نام میزبان و هویت (هستهٔ 1.7.0) ---
  'Upstream proxy': 'پروکسی بالادست',
  'Proxy address': 'نشانی پروکسی',
  'Aether dials out through this proxy — use it to chain behind another VPN or proxy already running on this PC. Empty = direct.':
    'اِتِر همهٔ اتصال‌های بیرونی‌اش را از این پروکسی می‌گیرد — برای زنجیره‌کردن پشت یک <bdi>VPN</bdi> یا پروکسیِ در حال اجرا روی همین ویندوز. خالی = اتصال مستقیم.',
  'That is not a proxy address Aether can use. Expected socks5://host:port or http://host:port — the port is required.':
    'این نشانی برای اِتِر قابل‌استفاده نیست. قالب درست: <bdi>socks5://host:port</bdi> یا <bdi>http://host:port</bdi> — نوشتن پورت الزامی است.',
  'An HTTP proxy cannot carry UDP, so MASQUE is switched to HTTP/2 automatically and WireGuard / WARP×2 will not pass through it. Use a SOCKS5 proxy for those.':
    'پروکسی <bdi>HTTP</bdi> نمی‌تواند <bdi>UDP</bdi> حمل کند؛ پس <bdi>MASQUE</bdi> خودکار روی <bdi>HTTP/2</bdi> می‌رود و <bdi>WireGuard</bdi> و <bdi>WARP×2</bdi> از این پروکسی رد نمی‌شوند. برای آن‌ها از پروکسی <bdi>SOCKS5</bdi> استفاده کنید.',
  'SOCKS5 with UDP support carries every protocol: MASQUE, WireGuard and WARP×2.':
    'پروکسی <bdi>SOCKS5</bdi> با پشتیبانی <bdi>UDP</bdi> هر سه پروتکل را حمل می‌کند: <bdi>MASQUE</bdi>، <bdi>WireGuard</bdi> و <bdi>WARP×2</bdi>.',
  'Match domain rules by real host name': 'تطبیق قواعد دامنه با نام واقعی میزبان',
  'Reads the name from the first bytes (TLS SNI or HTTP Host), so domain rules keep working even though Windows hands the tunnel an IP address':
    'نام میزبان را از بایت‌های اول (<bdi>TLS SNI</bdi> یا هدر <bdi>Host</bdi>) می‌خواند؛ پس قواعد دامنه حتی وقتی ویندوز فقط یک آی‌پی به تونل می‌دهد هم کار می‌کنند',
  'Account identity': 'هویت حساب',
  'Replace a refused identity': 'جایگزینی هویتِ ردشده',
  'If Cloudflare stops accepting the saved device, register a fresh one instead of handshaking a tunnel that carries no traffic':
    'اگر <bdi>Cloudflare</bdi> دیگر دستگاه ذخیره‌شده را نپذیرد، یک دستگاه تازه ثبت می‌شود؛ وگرنه تونل دست می‌دهد ولی هیچ ترافیکی عبور نمی‌کند',
  'The upstream proxy, host-name routing and identity replacement need engine core 1.7.0 or newer. The bundled core is older, so they are disabled.':
    'پروکسی بالادست، مسیریابی براساس نام میزبان و جایگزینی هویت به هستهٔ <bdi>1.7.0</bdi> یا بالاتر نیاز دارند. هستهٔ همراه این بیلد قدیمی‌تر است، پس غیرفعال شده‌اند.',

  'Reset to defaults': 'بازنشانی به تنظیمات پیش‌فرض',
  'Restores every setting above to its factory value': 'همهٔ تنظیمات بالا به مقدار کارخانه برمی‌گردد',
  'Reset': 'بازنشانی',

  // --- عیب‌یابی ---
  'Run the test to verify connectivity': 'برای بررسی اتصال، آزمایش را اجرا کنید',
  'A problem was detected — see the failing check': 'مشکلی پیدا شد — بررسیِ ناموفق را ببینید',
  'All checks passed — traffic should flow': 'همهٔ بررسی‌ها موفق بود — ترافیک باید برقرار باشد',
  'Testing connectivity…': 'در حال آزمایش اتصال…',
  'Run test': 'اجرای آزمایش',
  'Copy logs': 'کپی لاگ‌ها',
  'Clear': 'پاک‌سازی',
  'Environment check': 'بررسی محیط',
  'Log': 'لاگ',
  'No logs yet. Connect or run a test.': 'هنوز لاگی ثبت نشده. متصل شوید یا آزمایش را اجرا کنید.',
  'Logs copied to clipboard': 'لاگ‌ها در کلیپ‌بورد کپی شد',
  'Running…': 'در حال اجرا…',

  // --- اشتراک در شبکه ---
  'Other devices on the same Wi‑Fi can route their traffic through this computer. Point them at one of the addresses below.': 'دستگاه‌های دیگر روی همین <bdi>Wi‑Fi</bdi> می‌توانند ترافیکشان را از این رایانه عبور دهند. یکی از آدرس‌های زیر را در آن‌ها وارد کنید.',
  'Enable sharing': 'فعال‌سازی اشتراک',
  'Only listens on your local network address.': 'فقط روی آدرس شبکهٔ محلی شما گوش می‌دهد.',
  'Copy': 'کپی',
  'Sharing only works while Aether is connected.': 'اشتراک فقط وقتی کار می‌کند که <bdi>Aether</bdi> متصل باشد.',
  'Both ports accept HTTP and SOCKS5 automatically — either port works in either field.': 'هر دو پورت به‌صورت خودکار هم <bdi>HTTP</bdi> و هم <bdi>SOCKS5</bdi> را می‌پذیرند — هر پورتی را هر جا وارد کنید کار می‌کند.',
  'Apps like Telegram ignore the system proxy; set a SOCKS5 proxy inside the app instead.': 'برنامه‌هایی مثل تلگرام پروکسی سیستم را نادیده می‌گیرند؛ در تنظیمات خودِ برنامه یک پروکسی <bdi>SOCKS5</bdi> تنظیم کنید.',

  // --- درباره ---
  'App version': 'نسخهٔ برنامه',
  'Core version': 'نسخهٔ هسته',
  'Architecture': 'معماری',
  'Credits, links & what this build adds': 'سازندگان، لینک‌ها و امکانات این بیلد',
  'Version': 'نسخه',
  'Original project — Cluvex Studio': 'پروژهٔ اصلی — <bdi>Cluvex Studio</bdi>',
  'The core engine powering this app': 'موتور اصلیِ این برنامه',
  'Windows edition — QW-AI-Code': 'نسخهٔ ویندوز — <bdi>QW-AI-Code</bdi>',
  'The native Windows desktop edition of Aether — what we upgraded in this build': 'نسخهٔ بومی ویندوزِ <bdi>Aether</bdi> — بهبودهای همین بیلد',

  // --- ۱.۲.۴-p1: ذخیرهٔ کلید و تست اتصال ---
  'Show': 'نمایش',
  'Hide': 'پنهان',
  'API key saved.': 'کلید API ذخیره شد.',
  'API key removed': 'کلید API حذف شد',
  'Test the API connection': 'تست اتصال به API',
  'Testing…': 'در حال تست…',
  'Checks the key and lists the models it may use': 'کلید را بررسی می‌کند و مدل‌هایی که اجازهٔ استفاده دارد را فهرست می‌کند',
  'Working — {0} model(s) available through {1}': 'سالم — {0} مدل از مسیر {1} در دسترس است',
  'Not working: {0}': 'کار نمی‌کند: {0}',
}

let current = (() => {
  try {
    const v = localStorage.getItem(STORAGE_KEY)
    if (v === 'fa' || v === 'en') return v
  } catch { /* localStorage ممکن است در دسترس نباشد */ }
  return 'en'
})()

export function getLang() {
  return current
}

export function setLang(lang) {
  current = lang === 'fa' ? 'fa' : 'en'
  try {
    localStorage.setItem(STORAGE_KEY, current)
  } catch { /* بی‌اثر */ }
  applyLang()
}

/** جهت و فونت کل سند را با زبان فعلی هماهنگ می‌کند. */
export function applyLang() {
  const fa = current === 'fa'
  const html = document.documentElement
  html.lang = fa ? 'fa' : 'en'
  html.dir = fa ? 'rtl' : 'ltr'
  document.body.classList.toggle('lang-fa', fa)
}

/** ترجمهٔ یک رشتهٔ رابط — کلید = متن انگلیسی. */
/**
 * `<bdi>` را به جداسازهای دوسویهٔ یونیکد تبدیل می‌کند و هر تگِ دیگر را دور
 * می‌ریزد.
 *
 * # چرا این تابع وجود دارد
 *
 * ترجمه‌های فارسی برای محافظت از تکه‌های لاتین (`chrome.exe`، `SOCKS5`) از
 * `<bdi>` استفاده می‌کنند، و `app.css` هم قاعده‌ای برایش دارد. ولی همهٔ نماها
 * متن را با `textContent` می‌نشانند — که تنها راه درست است، چون یک رشتهٔ ترجمه
 * هرگز نباید به‌عنوان HTML اجرا شود. نتیجه این بود که کاربر عیناً
 * `<bdi>chrome.exe</bdi>` را روی صفحه می‌دید.
 *
 * `U+2068 FIRST STRONG ISOLATE` و `U+2069 POP DIRECTIONAL ISOLATE` دقیقاً همان
 * کاری را می‌کنند که `<bdi>` می‌کرد — جداسازیِ دوسویه — ولی نویسه‌اند و نه
 * نشانه‌گذاری، پس در `textContent` هم کار می‌کنند و هیچ راهی برای تزریق باز
 * نمی‌کنند.
 */
export function isolateBidi(text) {
  if (typeof text !== 'string' || !text.includes('<')) return text
  return text
    .replace(/<bdi>/g, '\u2068')
    .replace(/<\/bdi>/g, '\u2069')
    // هر تگِ دیگری که از قلم افتاده باشد پاک می‌شود: به کاربر نشان دادنِ
    // `<b>` بهتر از اجرایش نیست.
    .replace(/<[^>]*>/g, '')
}

export function t(key) {
  const raw = current === 'fa' ? (FA[key] ?? key) : key
  return isolateBidi(raw)
}
