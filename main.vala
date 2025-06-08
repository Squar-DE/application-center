using Gtk;
using Adw;

public class PackageInfo : Object {
    public string name { get; set; }
    public string version { get; set; }
    public string description { get; set; }
    public bool installed { get; set; }
    public int relevance_score { get; set; }
    
    public PackageInfo(string name, string version, string description, bool installed, int score = 0) {
        this.name = name;
        this.version = version;
        this.description = description;
        this.installed = installed;
        this.relevance_score = score;
    }
}

public class PacmanGui : Adw.Application {
    private Gtk.Stack                   updates_stack;
    private Gtk.FlowBox                 updates_results_flow;
    private Adw.ApplicationWindow       window;
    private Gtk.SearchEntry             search_entry;
    private Gtk.FlowBox                 results_flow;
    private Gtk.Stack                   main_stack;
    private Gtk.Spinner                 spinner;
    private Adw.Leaflet                 main_leaflet;
    private Gtk.Box                     info_panel;
    private Gtk.Label                   info_title;
    private Gtk.Label                   info_version;
    private Gtk.Label                   info_description;
    private Gtk.Button                  action_button;
    private PackageInfo?                selected_package          = null;
    private Gtk.Revealer                info_revealer;
    private Adw.ViewStack               nav_stack;
    private Adw.ViewSwitcherBar         nav_bar;
    private Adw.ToolbarView             toolbar_view;

    
    public PacmanGui() {
        Object(application_id: "com.example.pacmangui");
    }
    
    protected override void activate() {
        build_ui();
        window.present();
    }
   

    private void build_ui() {
        window = new Adw.ApplicationWindow(this);
        window.set_title("Pacman Package Manager");
        window.set_default_size(800, 600);
    
        // Outer layout
        toolbar_view = new Adw.ToolbarView();

        var header_bar = new Adw.HeaderBar();
        toolbar_view.add_top_bar(header_bar);

        // Pages
        var home = home_page();  // Now returns Gtk.Widget
        var updates = create_updates_page();
        var settings = new Gtk.Label("Settings page coming soon");

        nav_stack = new Adw.ViewStack();
        nav_stack.add_titled(home, "home", "Home").set_icon_name("go-home-symbolic");
        nav_stack.add_titled(updates, "updates", "Updates").set_icon_name("system-software-update-symbolic");
        nav_stack.add_titled(settings, "settings", "Settings").set_icon_name("emblem-system-symbolic");
        nav_stack.notify["visible-child"].connect(() => {
          var current = nav_stack.get_visible_child_name();
          if (current == "updates") {
            load_updates_async.begin();
          }
        }); 
        nav_bar = new Adw.ViewSwitcherBar();
        nav_bar.set_stack(nav_stack);
        nav_bar.set_reveal(true);

        toolbar_view.set_content(nav_stack);     // CORRECT: one content for toolbar_view
        toolbar_view.add_bottom_bar(nav_bar);

        window.set_content(toolbar_view);
    }

    private Gtk.Widget home_page() { 

        
        // Main content
        var main_box = new Gtk.Box(Gtk.Orientation.VERTICAL, 12);
        main_box.set_margin_top(12);
        main_box.set_margin_bottom(12);
        main_box.set_margin_start(12);
        main_box.set_margin_end(12);
        main_box.set_vexpand(true);
        
        // Search section
        var search_box = new Gtk.Box(Gtk.Orientation.HORIZONTAL, 6);
        
        search_entry = new Gtk.SearchEntry();
        search_entry.set_placeholder_text("Search packages...");
        search_entry.set_hexpand(true);
        search_entry.search_changed.connect(on_search_changed);
        search_entry.activate.connect(on_search_activate);
        
        var search_button = new Gtk.Button.with_label("Search");
        search_button.add_css_class("suggested-action");
        search_button.clicked.connect(on_search_activate);
        
        search_box.append(search_entry);
        search_box.append(search_button);
        
        // Stack for different states
        main_stack = new Gtk.Stack();
        main_stack.set_vexpand(true);
        
        // Welcome page
        var welcome_page = create_welcome_page();
        main_stack.add_named(welcome_page, "welcome");
        
        // Loading page
        var loading_page = create_loading_page();
        main_stack.add_named(loading_page, "loading");
        
        // Results page
        var results_page = create_results_page();
        main_stack.add_named(results_page, "results");
        
        // Empty results page
        var empty_page = create_empty_page();
        main_stack.add_named(empty_page, "empty");
        
        main_stack.set_visible_child_name("welcome");
        
        main_box.append(search_box);
        main_box.append(main_stack);
        
        // Create leaflet for sliding panel
        main_leaflet = new Adw.Leaflet();
        main_leaflet.set_can_navigate_back(true);
        main_leaflet.set_can_navigate_forward(true);
        main_leaflet.set_transition_type(Adw.LeafletTransitionType.SLIDE);
        
        var main_page = new Gtk.Box(Gtk.Orientation.VERTICAL, 0);
        main_page.append(main_box);

        info_panel = create_info_panel();
        info_revealer = new Gtk.Revealer ();
        info_revealer.transition_type = Gtk.RevealerTransitionType.SLIDE_LEFT;
        info_revealer.transition_duration = 300;
        info_revealer.child = info_panel;
        main_leaflet.append(main_page);
        main_leaflet.append(info_revealer); 
        main_leaflet.set_visible_child(main_page);
        
        return main_leaflet;
    }
    
    private Gtk.Widget create_welcome_page() {
        var box = new Gtk.Box(Gtk.Orientation.VERTICAL, 24);
        box.set_valign(Gtk.Align.CENTER);
        box.set_halign(Gtk.Align.CENTER);
        
        var icon = new Gtk.Image.from_icon_name("application-x-executable-symbolic");
        icon.set_icon_size(Gtk.IconSize.LARGE);
        icon.add_css_class("dim-label");
        
        var title = new Gtk.Label("Package Manager");
        title.add_css_class("title-1");
        
        var subtitle = new Gtk.Label("Search for packages to install or manage");
        subtitle.add_css_class("dim-label");
        
        box.append(icon);
        box.append(title);
        box.append(subtitle);
        
        return box;
    }
    
    private Gtk.Widget create_loading_page() {
        var box = new Gtk.Box(Gtk.Orientation.VERTICAL, 12);
        box.set_valign(Gtk.Align.CENTER);
        box.set_halign(Gtk.Align.CENTER);
        
        spinner = new Gtk.Spinner();
        spinner.set_spinning(true);
        spinner.set_size_request(48, 48);
        
        var label = new Gtk.Label("Searching packages...");
        label.add_css_class("dim-label");
        
        box.append(spinner);
        box.append(label);
        
        return box;
    }
    
    private Gtk.Widget create_results_page() {
        var box = new Gtk.Box(Gtk.Orientation.VERTICAL, 12);
        box.set_vexpand(true);
        
        var scrolled = new Gtk.ScrolledWindow();
        scrolled.set_policy(Gtk.PolicyType.NEVER, Gtk.PolicyType.AUTOMATIC);
        scrolled.set_vexpand(true);
        scrolled.set_min_content_height(300);
        
        results_flow = new Gtk.FlowBox();
        results_flow.set_max_children_per_line(3);
        results_flow.set_min_children_per_line(1);
        results_flow.set_column_spacing(12);
        results_flow.set_row_spacing(12);
        results_flow.set_selection_mode(Gtk.SelectionMode.SINGLE);
        results_flow.set_homogeneous(true);
        results_flow.child_activated.connect(on_package_activated);
        
        scrolled.set_child(results_flow);
        box.append(scrolled);
        
        return box;
    }
    
    private Gtk.Box create_info_panel() {
        var box = new Gtk.Box(Gtk.Orientation.VERTICAL, 12);
        box.set_margin_top(12);
        box.set_margin_bottom(12);
        box.set_margin_start(12);
        box.set_margin_end(12);

        // 🔙 Close button
        var close_button = new Gtk.Button.from_icon_name("window-close-symbolic");
            close_button.set_valign(Gtk.Align.START);
            close_button.set_halign(Gtk.Align.END);
            close_button.add_css_class("flat");
            close_button.clicked.connect(() => {
            main_leaflet.navigate(Adw.NavigationDirection.BACK);
        });

        box.append(close_button);

        // 🔤 Package info labels
        info_title = new Gtk.Label("");
        info_title.set_wrap(true);
        info_title.set_xalign(0);
        info_title.set_css_classes({ "title-1" });

        info_version = new Gtk.Label("");
        info_version.set_xalign(0);
        info_description = new Gtk.Label("");
        info_description.set_wrap(true);
        info_description.set_xalign(0);

        box.append(info_title);
        box.append(info_version);
        box.append(info_description);

        action_button = new Gtk.Button();
        action_button.set_sensitive(false);
        box.append(action_button);

        return box;
    }

    
    private Gtk.Widget create_empty_page() {
        var box = new Gtk.Box(Gtk.Orientation.VERTICAL, 12);
        box.set_valign(Gtk.Align.CENTER);
        box.set_halign(Gtk.Align.CENTER);
        
        var icon = new Gtk.Image.from_icon_name("system-search-symbolic");
        icon.set_icon_size(Gtk.IconSize.LARGE);
        icon.add_css_class("dim-label");
        
        var label = new Gtk.Label("No packages found");
        label.add_css_class("title-2");
        
        var subtitle = new Gtk.Label("Try a different search term");
        subtitle.add_css_class("dim-label");
        
        box.append(icon);
        box.append(label);
        box.append(subtitle);
        
        return box;
    }
    
    private void on_search_changed() {
        // Optional: implement live search with debouncing
    }
    
    private void on_search_activate() {
        var query = search_entry.get_text().strip();
        if (query.length > 0) {
            search_packages_async.begin(query);
        }
    }
    
    private async void search_packages_async(string query) {
        main_stack.set_visible_child_name("loading");
        
        // Clear previous results
        var child = results_flow.get_first_child();
        while (child != null) {
            var next = child.get_next_sibling();
            results_flow.remove(child);
            child = next;
        }
        
        try {
            var packages = yield perform_search(query);
            
            if (packages.length == 0) {
                main_stack.set_visible_child_name("empty");
                return;
            }
            
            // Sort packages by relevance score using a simple bubble sort
            sort_packages_by_relevance(packages);
            
            foreach (var pkg in packages) {
                add_package_card(pkg);
            }
            
            main_stack.set_visible_child_name("results");
            
        } catch (Error e) {
            show_error_dialog("Search failed: " + e.message);
            main_stack.set_visible_child_name("welcome");
        }
    }
    
    private void sort_packages_by_relevance(PackageInfo[] packages) {
        // Simple bubble sort by relevance score (descending)
        for (int i = 0; i < packages.length - 1; i++) {
            for (int j = 0; j < packages.length - 1 - i; j++) {
                if (packages[j].relevance_score < packages[j + 1].relevance_score) {
                    // Swap packages
                    var temp = packages[j];
                    packages[j] = packages[j + 1];
                    packages[j + 1] = temp;
                }
            }
        }
    }
    
    private async PackageInfo[] perform_search(string query) throws Error {
        return yield execute_pacman_command({"pacman", "-Ss", query}, query);
    }
    
    private async PackageInfo[] execute_pacman_command(string[] command, string? search_query = null) throws Error {
        var packages = new PackageInfo[0];
        
        try {
            string output;
            int exit_status;
            
            var subprocess = new Subprocess.newv(command, 
                SubprocessFlags.STDOUT_PIPE | SubprocessFlags.STDERR_MERGE);
            
            yield subprocess.communicate_utf8_async(null, null, out output, null);
            exit_status = subprocess.get_exit_status();
            
            if (exit_status != 0) {
                throw new IOError.FAILED("Command failed with exit code " + exit_status.to_string());
            }
            
            // Get installed packages for comparison
            var installed_packages = yield get_installed_packages();
            
            packages = parse_search_output(output, installed_packages, search_query);
            
        } catch (Error e) {
            throw e;
        }
        
        return packages;
    }
    
    private async string[] get_installed_packages() throws Error {
        try {
            string output;
            var subprocess = new Subprocess.newv({"pacman", "-Q"}, 
                SubprocessFlags.STDOUT_PIPE | SubprocessFlags.STDERR_MERGE);
            
            yield subprocess.communicate_utf8_async(null, null, out output, null);
            
            var installed = new string[0];
            var lines = output.split("\n");
            
            foreach (var line in lines) {
                var parts = line.strip().split(" ");
                if (parts.length >= 1 && parts[0].length > 0) {
                    installed += parts[0];
                }
            }
            
            return installed;
            
        } catch (Error e) {
            return new string[0];
        }
    }
    
    private PackageInfo[] parse_search_output(string output, string[] installed_packages, string? search_query = null) {
        var packages = new PackageInfo[0];
        var lines = output.split("\n");
        
        for (int i = 0; i < lines.length; i++) {
            var line = lines[i].strip();
            if (line.length == 0) continue;
            
            // Parse pacman -Ss output format: repo/name version [installed]
            if (line.contains("/")) {
                var parts = line.split(" ");
                if (parts.length >= 2) {
                    var name_part = parts[0].split("/");
                    if (name_part.length >= 2) {
                        var name = name_part[1];
                        var version = parts[1];
                        
                        // Check if installed
                        bool installed = line.contains("[installed]") || 
                                       is_package_installed(name, installed_packages);
                        
                        // Get description from next line if available
                        var description = "";
                        if (i + 1 < lines.length) {
                            var next_line = lines[i + 1].strip();
                            if (!next_line.contains("/")) {
                                description = next_line;
                            }
                        }
                        
                        // Calculate relevance score
                        int score = calculate_relevance_score(name, description, search_query);
                        
                        packages += new PackageInfo(name, version, description, installed, score);
                    }
                }
            }
        }
        
        return packages;
    }
    
    private int calculate_relevance_score(string package_name, string description, string? query) {
        if (query == null || query.length == 0) {
            return 0;
        }
        
        var query_lower = query.down();
        var name_lower = package_name.down();
        var desc_lower = description.down();
        
        int score = 0;
        
        // Exact name match gets highest score
        if (name_lower == query_lower) {
            score += 1000;
        }
        // Name starts with query gets high score
        else if (name_lower.has_prefix(query_lower)) {
            score += 800;
        }
        // Name contains query gets medium score
        else if (name_lower.contains(query_lower)) {
            score += 600;
        }
        
        // Description matches get lower scores
        if (desc_lower.contains(query_lower)) {
            score += 200;
        }
        
        // Bonus for shorter names (more specific matches)
        if (package_name.length <= 10) {
            score += 100;
        }
        
        // Penalty for very long names with dashes/underscores (usually extensions/plugins)
        if (package_name.length > 20 && (package_name.contains("-") || package_name.contains("_"))) {
            score -= 50;
        }
        
        // Bonus for exact word boundaries in name
        var words = name_lower.split("-");
        foreach (var word in words) {
            if (word == query_lower) {
                score += 300;
                break;
            }
        }
        
        return score;
    }
    
    private bool is_package_installed(string package_name, string[] installed_packages) {
        foreach (var installed in installed_packages) {
            if (installed == package_name) {
                return true;
            }
        }
        return false;
    }
    
    private void add_package_card(PackageInfo package) {
        var card = new Gtk.Box(Gtk.Orientation.VERTICAL, 8);
        card.add_css_class("card");
        card.set_size_request(200, 120);
        
        var content_box = new Gtk.Box(Gtk.Orientation.VERTICAL, 6);
        content_box.set_margin_top(12);
        content_box.set_margin_bottom(12);
        content_box.set_margin_start(12);
        content_box.set_margin_end(12);
        
        // Package icon
        var icon = new Gtk.Image.from_icon_name("application-x-executable-symbolic");
        icon.set_icon_size(Gtk.IconSize.NORMAL);
        icon.set_halign(Gtk.Align.CENTER);
        
        // Package name
        var name_label = new Gtk.Label(package.name);
        name_label.add_css_class("heading");
        name_label.set_halign(Gtk.Align.CENTER);
        name_label.set_ellipsize(Pango.EllipsizeMode.END);
        name_label.set_max_width_chars(20);
        
        // Version
        var version_label = new Gtk.Label(package.version);
        version_label.add_css_class("dim-label");
        version_label.add_css_class("caption");
        version_label.set_halign(Gtk.Align.CENTER);
        version_label.set_ellipsize(Pango.EllipsizeMode.END);
        
        // Install status indicator
        if (package.installed) {
            var status_box = new Gtk.Box(Gtk.Orientation.HORIZONTAL, 4);
            status_box.set_halign(Gtk.Align.CENTER);
            
            var installed_icon = new Gtk.Image.from_icon_name("object-select-symbolic");
            installed_icon.add_css_class("success");
            
            var installed_label = new Gtk.Label("Installed");
            installed_label.add_css_class("success");
            installed_label.add_css_class("caption");
            
            status_box.append(installed_icon);
            status_box.append(installed_label);
            content_box.append(status_box);
        }
        
        content_box.append(icon);
        content_box.append(name_label);
        content_box.append(version_label);
        
        card.append(content_box);
        
        // Store package info as data
        card.set_data("package", package);
        
        // Make it clickable
        var gesture = new Gtk.GestureClick();
        gesture.released.connect((n_press, x, y) => {
            selected_package = package;
            show_package_details(package);
        });
        card.add_controller(gesture);
        
        // Add hover effect
        var motion_controller = new Gtk.EventControllerMotion();
        motion_controller.enter.connect(() => {
            card.add_css_class("card-hover");
        });
        motion_controller.leave.connect(() => {
            card.remove_css_class("card-hover");
        });
        card.add_controller(motion_controller);
        
        results_flow.append(card);
    }
    
    private void show_package_details(PackageInfo package) {
        selected_package = package;
    
        // Update the info panel content
        info_title.set_text(package.name);
        info_version.set_text("Version: " + package.version);
    
        var description_text = package.description.length > 0 ? package.description : "No description available.";
        info_description.set_text(description_text);

        action_button.set_sensitive(true);
        if (package.installed) {
            action_button.set_label("Uninstall");
            action_button.remove_css_class("suggested-action");
            action_button.add_css_class("destructive-action");
        } else {
            action_button.set_label("Install");
            action_button.remove_css_class("destructive-action");
            action_button.add_css_class("suggested-action");
        }

        info_revealer.reveal_child = true;

        main_leaflet.navigate(Adw.NavigationDirection.FORWARD);
        action_button.clicked.disconnect(on_action_clicked);
        action_button.clicked.connect(on_action_clicked);
    }
    
    private void on_package_activated(Gtk.FlowBoxChild child) {
        var card = child.get_first_child();
        if (card != null) {
            var package = card.get_data<PackageInfo>("package");
            if (package != null) {
                show_package_details(package);
            }
        }
    }
    
    private void on_action_clicked() {
    if (selected_package == null)
        return;

    if (selected_package.installed) {
        show_confirmation_dialog(
            "Uninstall " + selected_package.name + "?",
            "This will remove the package from your system.",
            () => {
                on_confirm_uninstall.begin();
            }
        );
    } else {
        install_package_async.begin(selected_package.name);
    }
}
    async void on_confirm_uninstall() {
        yield uninstall_package_async(selected_package.name);
    }
 
    private async void install_package_async(string package_name) {
        try {
            action_button.set_sensitive(false);
            action_button.set_label("Installing...");
            
            var subprocess = new Subprocess.newv({"pkexec", "pacman", "-S", "--noconfirm", package_name}, 
                SubprocessFlags.STDOUT_PIPE | SubprocessFlags.STDERR_MERGE);
            
            string output;
            yield subprocess.communicate_utf8_async(null, null, out output, null);
            int exit_status = subprocess.get_exit_status();
            
            if (exit_status == 0) {
                show_toast("Package installed successfully");
                selected_package.installed = true;
                refresh_current_view();
            } else {
                show_error_dialog("Installation failed: " + output);
            }
            
        } catch (Error e) {
            show_error_dialog("Installation failed: " + e.message);
        } finally {
            update_action_button();
        }
    }
    
    private async void uninstall_package_async(string package_name) {
        try {
            action_button.set_sensitive(false);
            action_button.set_label("Uninstalling...");
            
            var subprocess = new Subprocess.newv({"pkexec", "pacman", "-R", "--noconfirm", package_name}, 
                SubprocessFlags.STDOUT_PIPE | SubprocessFlags.STDERR_MERGE);
            
            string output;
            yield subprocess.communicate_utf8_async(null, null, out output, null);
            int exit_status = subprocess.get_exit_status();
            
            if (exit_status == 0) {
                show_toast("Package uninstalled successfully");
                selected_package.installed = false;
                refresh_current_view();
            } else {
                show_error_dialog("Uninstallation failed: " + output);
            }
            
        } catch (Error e) {
            show_error_dialog("Uninstallation failed: " + e.message);
        } finally {
            update_action_button();
        }
    }
    
    private void refresh_current_view() {
        if (selected_package != null) {
            // Update the info panel
            show_package_details(selected_package);
            
            // Update the card in the flow box
            var child = results_flow.get_first_child();
            while (child != null) {
                var card = child.get_first_child();
                if (card != null) {
                    var package = card.get_data<PackageInfo>("package");
                    if (package != null && package.name == selected_package.name) {
                        // Rebuild this card
                        results_flow.remove(child);
                        add_package_card(selected_package);
                        break;
                    }
                }
                child = child.get_next_sibling();
            }
        }
    }
    
    private void update_action_button() {
        if (selected_package != null) {
            action_button.set_sensitive(true);
            
            if (selected_package.installed) {
                action_button.set_label("Uninstall");
                action_button.remove_css_class("suggested-action");
                action_button.add_css_class("destructive-action");
            } else {
                action_button.set_label("Install");
                action_button.remove_css_class("destructive-action");
                action_button.add_css_class("suggested-action");
            }
        }
    }

    private Gtk.Widget create_updates_page() {
    var box = new Gtk.Box(Gtk.Orientation.VERTICAL, 12);
    box.set_vexpand(true);

    // Header row with refresh and "Update All" (optional)
    var header_box = new Gtk.Box(Gtk.Orientation.HORIZONTAL, 6);

var check_updates_button = new Gtk.Button.with_label("Check for Updates");
check_updates_button.add_css_class("suggested-action");
check_updates_button.clicked.connect(() => { check_for_updates_async.begin(); });
header_box.append(check_updates_button);

var update_all_button = new Gtk.Button.with_label("Update All");
update_all_button.add_css_class("suggested-action");
update_all_button.clicked.connect(() => { update_all_packages_async.begin(); });
header_box.append(update_all_button);

box.append(header_box);

    // Stack for loading/results/empty
    updates_stack = new Gtk.Stack();
    updates_stack.set_vexpand(true);

    var loading = create_loading_page();
    updates_stack.add_named(loading, "loading");

    var empty = create_empty_page();
    updates_stack.add_named(empty, "empty");

    updates_results_flow = new Gtk.FlowBox();
    updates_results_flow.set_max_children_per_line(3);
    updates_results_flow.set_column_spacing(12);
    updates_results_flow.set_row_spacing(12);

    var results_scroll = new Gtk.ScrolledWindow();
    results_scroll.set_child(updates_results_flow);
    updates_stack.add_named(results_scroll, "results");

    box.append(updates_stack);

    return box;
}


private async void check_for_updates_async() {
    updates_stack.set_visible_child_name("loading");
    try {
        // First, sync package databases
        var sync_proc = new Subprocess.newv({"pkexec", "pacman", "-Sy"},
            SubprocessFlags.STDOUT_PIPE | SubprocessFlags.STDERR_MERGE);
        string sync_output;
        yield sync_proc.communicate_utf8_async(null, null, out sync_output, null);
        int sync_status = sync_proc.get_exit_status();

        if (sync_status != 0) {
            show_error_dialog("Failed to refresh package database:\n" + sync_output);
            updates_stack.set_visible_child_name("empty");
            return;
        }

        // Then, load updates as usual
        yield load_updates_async();
    } catch (Error e) {
        show_error_dialog("Failed to check for updates:\n" + e.message);
        updates_stack.set_visible_child_name("empty");
    }
}

private async void load_updates_async() {
    updates_stack.set_visible_child_name("loading");
    // Clear previous results
    var child = updates_results_flow.get_first_child();
    while (child != null) {
        var next = child.get_next_sibling();
        updates_results_flow.remove(child);
        child = next;
    }

    try {
        var packages = yield get_upgradable_packages();
        if (packages.length == 0) {
            updates_stack.set_visible_child_name("empty");
            return;
        }
        foreach (var pkg in packages) {
            add_update_package_card(pkg);
        }
        updates_stack.set_visible_child_name("results");
    } catch (Error e) {
        show_error_dialog("Failed to load updates:\n" + e.message);
        updates_stack.set_visible_child_name("empty");
    }
}


private void add_update_package_card(PackageInfo package) {
    var card = new Gtk.Box(Gtk.Orientation.VERTICAL, 8);
    card.add_css_class("card");
    card.set_size_request(200, 120);

    var content_box = new Gtk.Box(Gtk.Orientation.VERTICAL, 6);
    content_box.set_margin_top(12);
    content_box.set_margin_bottom(12);
    content_box.set_margin_start(12);
    content_box.set_margin_end(12);

    // Package icon
    var icon = new Gtk.Image.from_icon_name("system-software-update-symbolic");
    icon.set_icon_size(Gtk.IconSize.NORMAL);
    icon.set_halign(Gtk.Align.CENTER);

    // Package name
    var name_label = new Gtk.Label(package.name);
    name_label.add_css_class("heading");
    name_label.set_halign(Gtk.Align.CENTER);
    name_label.set_ellipsize(Pango.EllipsizeMode.END);
    name_label.set_max_width_chars(20);

    // Parse version info: "old_version → new_version"
    string old_version = "", new_version = "";
    var versions = package.description.split("→");
    if (versions.length == 2) {
        old_version = versions[0].strip();
        new_version = versions[1].strip();
    }

    // Version row (old_version → new_version)
    var version_row = new Gtk.Box(Gtk.Orientation.HORIZONTAL, 4);
    version_row.set_halign(Gtk.Align.CENTER);

    var old_label = new Gtk.Label(old_version);
    old_label.add_css_class("dim-label");

    var arrow_label = new Gtk.Label("→");
    arrow_label.add_css_class("dim-label");

    var new_label = new Gtk.Label(new_version);
    new_label.add_css_class("success"); // This will make it green according to Adwaita/Libadwaita

    version_row.append(old_label);
    version_row.append(arrow_label);
    version_row.append(new_label);

    // Update button
    var update_button = new Gtk.Button.with_label("Update");
    update_button.add_css_class("suggested-action");
    update_button.clicked.connect(() => {
        update_package_async.begin(package.name);
    });

    content_box.append(icon);
    content_box.append(name_label);
    content_box.append(version_row);
    content_box.append(update_button);

    card.append(content_box);

    // Hover effect
    var motion_controller = new Gtk.EventControllerMotion();
    motion_controller.enter.connect(() => {
        card.add_css_class("card-hover");
    });
    motion_controller.leave.connect(() => {
        card.remove_css_class("card-hover");
    });
    card.add_controller(motion_controller);

    updates_results_flow.append(card);
}

private async PackageInfo[] get_upgradable_packages() throws Error {
    var packages = new PackageInfo[0];
    string output;
    int exit_status;

    var subprocess = new Subprocess.newv({"pacman", "-Qu"},
        SubprocessFlags.STDOUT_PIPE | SubprocessFlags.STDERR_MERGE);

    yield subprocess.communicate_utf8_async(null, null, out output, null);
    exit_status = subprocess.get_exit_status();

    // Treat exit code 1 as "no updates"
    if (exit_status == 1) {
        return packages;
    }
    if (exit_status != 0) {
        throw new IOError.FAILED("pacman -Qu failed (" + exit_status.to_string() + ")");
    }

    var lines = output.split("\n");
    foreach (var line in lines) {
        var parts = line.strip().split(" ");
        if (parts.length >= 4 && parts[2] == "->") {
            var name = parts[0];
            var old_version = parts[1];
            var new_version = parts[3];
            var desc = old_version + " → " + new_version;
            packages += new PackageInfo(name, new_version, desc, true);
        }
    }
    return packages;
}


private async void update_package_async(string package_name) {
    try {
        var subprocess = new Subprocess.newv({"pkexec", "pacman", "-S", "--noconfirm", package_name},
            SubprocessFlags.STDOUT_PIPE | SubprocessFlags.STDERR_MERGE);
        string output;
        yield subprocess.communicate_utf8_async(null, null, out output, null);
        int exit_status = subprocess.get_exit_status();
        if (exit_status == 0) {
            show_toast("Updated " + package_name + " successfully");
            load_updates_async.begin();
        } else {
            show_error_dialog("Failed to update " + package_name + ":\n" + output);
        }
    } catch (Error e) {
        show_error_dialog("Failed to update " + package_name + ":\n" + e.message);
    }
}

private async void update_all_packages_async() {
    try {
        var subprocess = new Subprocess.newv({"pkexec", "pacman", "-Syu", "--noconfirm"},
            SubprocessFlags.STDOUT_PIPE | SubprocessFlags.STDERR_MERGE);
        string output;
        yield subprocess.communicate_utf8_async(null, null, out output, null);
        int exit_status = subprocess.get_exit_status();
        if (exit_status == 0) {
            show_toast("All packages updated successfully");
            load_updates_async.begin();
        } else {
            show_error_dialog("Failed to update all packages:\n" + output);
        }
    } catch (Error e) {
        show_error_dialog("Failed to update all packages:\n" + e.message);
    }
}
    
    private bool confirmation_dialog_open = false;

private void show_confirmation_dialog(string title, string message, owned Func callback) {
    if (confirmation_dialog_open)
        return;  // Prevent multiple dialogs

    confirmation_dialog_open = true;

    var dialog = new Adw.MessageDialog(window, title, message);
    dialog.set_heading(title);
    dialog.set_body(message);
    dialog.add_response("cancel", "Cancel");
    dialog.add_response("confirm", "Confirm");
    dialog.set_response_appearance("confirm", Adw.ResponseAppearance.DESTRUCTIVE);
    dialog.set_default_response("cancel");
    dialog.set_close_response("cancel");

    dialog.response.connect((response) => {
        if (response == "confirm") {
            callback();
        }

        confirmation_dialog_open = false;
        dialog.destroy();  // Always destroy
    });

    dialog.present();
}

    
    private void show_error_dialog(string message) {
        var dialog = new Adw.MessageDialog(window, "Error", message);
        dialog.add_response("ok", "OK");
        dialog.set_default_response("ok");
        dialog.set_close_response("ok");
        dialog.present();
    }
    
    private void show_toast(string message) {
        var toast = new Adw.Toast(message);
        toast.set_timeout(3);
        
        // Note: In a real app, you'd want to add this to an Adw.ToastOverlay
        // For simplicity, we'll use a simple message dialog here
        var dialog = new Adw.MessageDialog(window, "Success", message);
        dialog.add_response("ok", "OK");
        dialog.set_default_response("ok");
        dialog.set_close_response("ok");
        dialog.present();
    }
    
    public static int main(string[] args) {
        var app = new PacmanGui();
        return app.run(args);
    }
}

// Delegate type for callbacks
public delegate void Func();
