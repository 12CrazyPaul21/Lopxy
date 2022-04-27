/**
 * lopxy web manager logical module
 */

const lopxy = (function() {

    function lopxy_init_failed() {
        $("#lopxy-init-status").text("Lopxy Init Failed");
    }

    function into_lopxy_shutdown_page() {
        $("#lopxy-web-manager-container").fadeOut(500, () => {
            $("#lopxy-web-manager-container").remove();
            $("#lopxy-shutdown-screen").removeClass("lopxy-hide");
        });
    }

    function into_lopxy_web_manager_page() {
        let self = this;

        //
        // lopxy server status
        //

        $("#lopxy-confirm-shutdown-btn").on('click', function() {
            $('#lopxy-shutdown-dialog').on('hidden.bs.modal', function() {
                $('#lopxy-shutdown-dialog').off('hidden.bs.modal');
                self.shutdown_all_server();
            });
            $("#lopxy-shutdown-dialog").modal('hide');
        });

        $("#lopxy-enable-proxy").on('click', function() {
            self.enable_lopxy_proxy();
        });

        $("#lopxy-disable-proxy").on('click', function() {
            self.disable_lopxy_proxy();
        });

        //
        // proxy item
        //

        $("#lopxy-confirm-add-proxy-item-btn").on('click', function() {
            $("#lopxy-add-proxy-item-dialog").modal('hide');

            let newItemInfo = {
                resource_url: $("#lopxy-add-proxy-item-resource-url").val(),
                proxy_resource_url: $("#lopxy-add-proxy-item-proxy-resource-url").val(),
                content_type: $("#lopxy-add-proxy-item-content-type").val()
            };

            $("#lopxy-add-proxy-item-resource-url").val('');
            $("#lopxy-add-proxy-item-proxy-resource-url").val('');
            $("#lopxy-add-proxy-item-content-type").val('application/octet-stream');

            if (!newItemInfo.resource_url.length || !newItemInfo.proxy_resource_url.length || !newItemInfo.content_type.length) {
                return;
            }

            self.add_lopxy_proxy_item(newItemInfo);
        });

        $(document).on('click', '.lopxy-modify-proxy-item-btn', function() {
            let children = $(this).parent().parent().children();

            let resourceUrl = $(children[0]).text();
            let proxyResourceUrl = $(children[1]).text();
            let contentType = $(children[2]).text();

            $("#lopxy-modify-proxy-item-resource-url").text(resourceUrl);
            $("#lopxy-modify-proxy-item-proxy-resource-url").val(proxyResourceUrl);
            $("#lopxy-modify-proxy-item-content-type").val(contentType);

            $("#lopxy-modify-proxy-item-dialog").attr("target-proxy-item-name", resourceUrl);
            $("#lopxy-modify-proxy-item-dialog").modal('show');
        });

        $("#lopxy-confirm-modify-proxy-item-btn").on('click', function() {
            $("#lopxy-modify-proxy-item-dialog").modal('hide');

            let newItemInfo = {
                resource_url: $("#lopxy-modify-proxy-item-resource-url").text(),
                proxy_resource_url: $("#lopxy-modify-proxy-item-proxy-resource-url").val(),
                content_type: $("#lopxy-modify-proxy-item-content-type").val()
            };

            let itemTag = get_proxy_item(newItemInfo.resource_url);
            let itemInfo = get_proxy_item_info(newItemInfo.resource_url);

            if (itemTag == null || itemInfo == null) {
                return;
            }

            if (newItemInfo.proxy_resource_url == itemInfo.proxy_resource_url && newItemInfo.content_type == itemInfo.content_type) {
                return;
            }

            self.modify_lopxy_proxy_item(itemTag, newItemInfo);
        });

        $(document).on('click', '.lopxy-remove-proxy-item-btn', function() {
            let targetProxyItemName = $(this).parent().parent().attr("proxy-item");
            $("#lopxy-remove-proxy-item-tips-dialog").attr("target-proxy-item-name", targetProxyItemName);
            $("#lopxy-remove-proxy-item-tips-dialog").modal('show');
        });

        $("#lopxy-confirm-remove-proxy-item-btn").on('click', function() {
            $("#lopxy-remove-proxy-item-tips-dialog").modal('hide');
            let targetProxyItemName = $("#lopxy-remove-proxy-item-tips-dialog").attr("target-proxy-item-name");
            self.remove_lopxy_proxy_item(targetProxyItemName);
        });

        //
        // request status
        //

        $('#lopxy-clean-request-status-logs-btn').on('click', function() {
            $("#lopxy-request-status-logs-table tbody").empty();
        });

        $('#lopxy-refresh-request-status-logs-btn').on('click', function() {
            request_refresh_lopxy_status.bind(lopxy)();
        });

        $('#lopxy-auto-refresh-enabled-btn').on('click', function() {
            if (self.is_lopxy_status_monitor_started()) {
                self.stop_lopxy_status_monitor();
                $('#lopxy-auto-refresh-enabled-btn').text("开启自动刷新");
            } else {
                self.start_lopxy_status_monitor();
                $('#lopxy-auto-refresh-enabled-btn').text("关闭自动刷新");
            }
        });

        // into web manager container page
        $("#lopxy-splash-screen").fadeOut(1000, () => {
            $("#lopxy-splash-screen").remove();
            $("#lopxy-web-manager-container").removeClass("lopxy-hide");
        });
    }

    function create_request_status_log_item(item) {
        function getLocalTime(ms) {  
            return new Date(parseInt(ms)).toLocaleString().replace(/:\d{1,2}$/,' ');  
        }
        return `<tr>
    <td>${getLocalTime(item.timestamp)}</td>
    <td>${item.bin_name}</td>
    <td>${item.pid}</td>
    <td>${item.path}</td>
    <td>${item.status}</td>
</tr>`;
    }

    function append_request_status_logs(new_request_status_logs) {
        for (let item of new_request_status_logs) {
            $("#lopxy-request-status-logs-table tbody").append(create_request_status_log_item(item));
        }
    }

    function is_proxy_item_exist(proxyItems, resourceName) {
        for (let item of proxyItems) {
            if (item['resource_url'] == resourceName) {
                return true;
            }
        }

        return false;
    }

    function add_proxy_item(item) {
        $("#lopxy-proxy-items-table tbody").append(`<tr proxy-item="${item.resource_url}">
    <td>${item.resource_url}</td>
    <td>${item.proxy_resource_url}</td>
    <td>${item.content_type}</td>
    <td>
        <button type="button" class="btn btn-info btn-sm lopxy-modify-proxy-item-btn">修改</button>
        <button type="button" class="btn btn-danger btn-sm lopxy-remove-proxy-item-btn">删除</button>
    </td>
</tr>
        `);
    }

    function modify_proxy_item(item_tag, item) {
        let children = $(item_tag).children();
        $(children[0]).text(item['resource_url']);
        $(children[1]).text(item['proxy_resource_url']);
        $(children[2]).text(item['content_type']);
    }

    function get_proxy_item(itemName) {
        let item = $(`#lopxy-proxy-items-table tbody tr[proxy-item='${itemName}']`);
        if (item.length == 0) {
            return null;
        }
        return item[0];
    }

    function get_proxy_item_info(itemName) {
        let item = get_proxy_item(itemName);
        if (item == null) {
            return null;
        }

        let children = $(item).children();
        return {
            resource_url: $(children[0]).text(),
            proxy_resource_url: $(children[1]).text(),
            content_type: $(children[2]).text()
        };
    }

    function update_proxy_item_list(proxyItems) {
        for (let old_item of $("#lopxy-proxy-items-table tbody tr[proxy-item] td:first-child")) {
            let resourceName = $(old_item).text();
            if (!is_proxy_item_exist(proxyItems, resourceName)) {
                $(old_item).parent().remove();
            }
        }

        for (let item of proxyItems) {
            let old_item = $(`#lopxy-proxy-items-table tbody tr[proxy-item='${item.resource_url}']`);
            if (old_item.length == 0) {
                add_proxy_item(item);
            } else {
                modify_proxy_item(old_item, item);
            }
        }
    }

    function update_proxy_enabled_button_status(enabled) {
        if (enabled) {
            $("#lopxy-proxy-enabled").text("启用");
            $("#lopxy-enable-proxy").attr("disabled", true);
            $("#lopxy-disable-proxy").attr("disabled", false);
        } else {
            $("#lopxy-proxy-enabled").text("禁用");
            $("#lopxy-enable-proxy").attr("disabled", false);
            $("#lopxy-disable-proxy").attr("disabled", true);
        }
    }

    function update_lopxy_status(lopxy_status) {
        if (!lopxy_status['success']) {
            return false;
        }

        this.env.webManagerPort = lopxy_status['web_manager_port'];
        this.env.proxyPort = lopxy_status['proxy_port'];
        this.env.proxyEnabled = lopxy_status['proxy_enabled'];
        update_proxy_enabled_button_status(this.env.proxyEnabled);

        $("#lopxy-web-manager-port").text(this.env.webManagerPort);
        $("#lopxy-proxy-server-port").text(this.env.proxyPort);

        if (lopxy_status['updated']) {
            if (this.env.statusUpdateTimestamp != lopxy_status['status_log_timestamp']) {
                this.env.statusUpdateTimestamp = lopxy_status['status_log_timestamp'];
                let new_request_status_logs = lopxy_status['request_status_logs'];
                this.env.requestStatusLogs = this.env.requestStatusLogs.concat(new_request_status_logs);
                append_request_status_logs(new_request_status_logs);
            }

            if (this.env.configTimestamp != lopxy_status['config_timestamp']) {
                this.env.configTimestamp = lopxy_status['config_timestamp'];
                this.env.proxyItems = lopxy_status['proxy_items'];
                update_proxy_item_list(this.env.proxyItems);
            }
        }

        return true;
    }

    function request_refresh_lopxy_status() {
        $.ajax({
            type: 'get',
            url: '/status',
            data: {
                config_timestamp: this.env.configTimestamp,
                status_log_timestamp: this.env.statusUpdateTimestamp,
            },
            success: (lopxy_status) => {
                if (!lopxy_status['success']) {
                    return;
                }
                update_lopxy_status.bind(this)(lopxy_status);
            },
            error: () => {
            }
        });
    }

    function request_set_lopxy_proxy_enabled(enabled) {
        $.ajax({
            type: 'post',
            url: '/enable_proxy',
            data: {
                enabled: enabled,
            }
        });
    }

    function request_remove_lopxy_proxy_item(targetProxyItemName) {
        $.ajax({
            type: 'delete',
            url: '/remove',
            data: {
                resource: encodeURI(targetProxyItemName),
            }
        });
    }

    return {
        env: {
            webManagerPort: 0,
            proxyPort: 0,
            proxyEnabled: false,
            statusUpdateTimestamp: 0,
            requestStatusLogs: [],
            configTimestamp: 0,
            proxyItems: [],
            timer: null,
            refreshStatusInterval: 1000
        },
        init: function() {
            $.ajax({
                type: 'get',
                url: '/status',
                data: {
                    config_timestamp: this.env.configTimestamp,
                    status_log_timestamp: this.env.statusUpdateTimestamp,
                },
                success: (lopxy_status) => {
                    if (!lopxy_status['success']) {
                        lopxy_init_failed();
                        return;
                    }

                    $("#lopxy-init-status").text("Into Lopxy Web Manager Page");
                    update_lopxy_status.bind(this)(lopxy_status);
                    into_lopxy_web_manager_page.bind(this)();
                    this.start_lopxy_status_monitor();
                },
                error: () => {
                    lopxy_init_failed();
                }
            })
        },        
        shutdown_all_server: function() {
            this.stop_lopxy_status_monitor();
            $("#lopxy-shutdown-dialog-trigger").attr("disabled", true);
            $.ajax({
                type: 'get',
                url: '/shutdown',
                success: (lopxy_status) => {
                    into_lopxy_shutdown_page();
                },
                error: () => {
                    into_lopxy_shutdown_page();
                }
            });
        },
        start_lopxy_status_monitor: function() {
            if (this.env.timer) {
                clearInterval(this.env.timer);
            }
    
            this.env.timer = setInterval(request_refresh_lopxy_status.bind(this), this.env.refreshStatusInterval);
        },
        stop_lopxy_status_monitor: function() {
            if (this.env.timer) {
                clearInterval(this.env.timer);
                this.env.timer = null;
            }
        },
        is_lopxy_status_monitor_started: function() {
            return this.env.timer != null;
        },
        enable_lopxy_proxy: function() {
            this.env.proxyEnabled = true;
            request_set_lopxy_proxy_enabled(true);
            update_proxy_enabled_button_status(true);
        },
        disable_lopxy_proxy: function() {
            this.env.proxyEnabled = false;
            request_set_lopxy_proxy_enabled(false);
            update_proxy_enabled_button_status(false);
        },
        add_lopxy_proxy_item: function(newItem) {
            $.ajax({
                type: 'post',
                url: '/add',
                data: {
                    resource: encodeURI(newItem.resource_url),
                    resource_proxy: encodeURI(newItem.proxy_resource_url),
                    resource_content_type: newItem.content_type
                },
                success: (status) => {
                    if (!status.result) {
                        return;
                    }

                    add_proxy_item(newItem);
                }
            });
        },
        modify_lopxy_proxy_item: function(itemTag, newItemInfo) {
            $.ajax({
                type: 'post',
                url: '/modify',
                data: {
                    resource: encodeURI(newItemInfo.resource_url),
                    resource_proxy: encodeURI(newItemInfo.proxy_resource_url),
                    resource_content_type: newItemInfo.content_type
                },
                success: (status) => {
                    if (!status.result) {
                        return;
                    }

                    modify_proxy_item(itemTag, newItemInfo);
                }
            });
        },
        remove_lopxy_proxy_item: function(targetProxyItemName) {
            let targetProxyItem = $(`#lopxy-proxy-items-table tbody tr[proxy-item='${targetProxyItemName}']`);
            if (targetProxyItem.length == 0) {
                return;
            }

            targetProxyItem.remove();            
            request_remove_lopxy_proxy_item(targetProxyItemName);
        }
    }
})();