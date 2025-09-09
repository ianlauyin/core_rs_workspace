FROM opensearchproject/opensearch-dashboards:3
RUN /usr/share/opensearch-dashboards/bin/opensearch-dashboards-plugin remove securityDashboards
RUN /usr/share/opensearch-dashboards/bin/opensearch-dashboards-plugin remove alertingDashboards
RUN /usr/share/opensearch-dashboards/bin/opensearch-dashboards-plugin remove anomalyDetectionDashboards
RUN /usr/share/opensearch-dashboards/bin/opensearch-dashboards-plugin remove securityAnalyticsDashboards
RUN /usr/share/opensearch-dashboards/bin/opensearch-dashboards-plugin remove assistantDashboards
RUN /usr/share/opensearch-dashboards/bin/opensearch-dashboards-plugin remove customImportMapDashboards
RUN /usr/share/opensearch-dashboards/bin/opensearch-dashboards-plugin remove mlCommonsDashboards
RUN /usr/share/opensearch-dashboards/bin/opensearch-dashboards-plugin remove notificationsDashboards
RUN /usr/share/opensearch-dashboards/bin/opensearch-dashboards-plugin remove searchRelevanceDashboards
RUN /usr/share/opensearch-dashboards/bin/opensearch-dashboards-plugin remove reportsDashboards
RUN /usr/share/opensearch-dashboards/bin/opensearch-dashboards-plugin remove flowFrameworkDashboards
COPY --chown=opensearch-dashboards:opensearch-dashboards opensearch_dashboards.yml /usr/share/opensearch-dashboards/config/
