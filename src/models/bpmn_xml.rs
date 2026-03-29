/// BPMN XML Serialization (BPMNDI format)
///
/// Export BPMN diagrams to XML and import from XML according to BPMN 2.0 specification.
use crate::models::bpmn::*;
use crate::utils::common::escape_xml_string;

/// XML builder for BPMN
pub struct BPMNXmlBuilder;

impl BPMNXmlBuilder {
    /// Convert BPMN diagram to XML string
    pub fn to_xml(diagram: &BPMNDiagram) -> String {
        let mut xml = String::new();

        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str(&format!(
            "<bpmn:definitions xmlns:bpmn=\"http://www.omg.org/spec/BPMN/20100524/MODEL\" \
             xmlns:bpmndi=\"http://www.omg.org/spec/BPMN/20100524/DI\" \
             xmlns:dc=\"http://www.omg.org/spec/DD/20100524/DC\" \
             id=\"Definitions_{}\">",
            escape_xml_string(&diagram.id)
        ));
        xml.push('\n');

        // Process element
        xml.push_str(&format!(
            "  <bpmn:process id=\"Process_{}\">",
            escape_xml_string(&diagram.id)
        ));
        xml.push('\n');

        // Add all flow elements
        for event in diagram.events.values() {
            match event.event_type {
                EventType::Start => {
                    xml.push_str(&format!(
                        "    <bpmn:startEvent id=\"{}\" name=\"{}\"/>\n",
                        escape_xml_string(&event.id),
                        escape_xml_string(&event.name)
                    ));
                }
                EventType::End => {
                    xml.push_str(&format!(
                        "    <bpmn:endEvent id=\"{}\" name=\"{}\"/>\n",
                        escape_xml_string(&event.id),
                        escape_xml_string(&event.name)
                    ));
                }
                EventType::Intermediate => {
                    xml.push_str(&format!(
                        "    <bpmn:intermediateCatchEvent id=\"{}\" name=\"{}\"/>\n",
                        escape_xml_string(&event.id),
                        escape_xml_string(&event.name)
                    ));
                }
                EventType::Boundary => {
                    xml.push_str(&format!(
                        "    <bpmn:boundaryEvent id=\"{}\" name=\"{}\"/>\n",
                        escape_xml_string(&event.id),
                        escape_xml_string(&event.name)
                    ));
                }
            }
        }

        for task in diagram.tasks.values() {
            let task_element = match task.task_type {
                TaskType::UserTask => "userTask",
                TaskType::ServiceTask => "serviceTask",
                TaskType::AutomaticTask => "task",
                TaskType::ManualTask => "manualTask",
                TaskType::SendTask => "sendTask",
                TaskType::ReceiveTask => "receiveTask",
                TaskType::ScriptTask => "scriptTask",
            };

            xml.push_str(&format!(
                "    <bpmn:{} id=\"{}\" name=\"{}\"/>\n",
                task_element,
                escape_xml_string(&task.id),
                escape_xml_string(&task.name)
            ));
        }

        for gateway in diagram.gateways.values() {
            let gateway_element = match gateway.gateway_type {
                GatewayType::ExclusiveXor => "exclusiveGateway",
                GatewayType::Parallel => "parallelGateway",
                GatewayType::Inclusive => "inclusiveGateway",
                GatewayType::EventBased => "eventBasedGateway",
            };

            xml.push_str(&format!(
                "    <bpmn:{} id=\"{}\" name=\"{}\"/>\n",
                gateway_element,
                escape_xml_string(&gateway.id),
                escape_xml_string(&gateway.name)
            ));
        }

        // Sequence flows
        for flow in diagram.flows.values() {
            xml.push_str(&format!(
                "    <bpmn:sequenceFlow id=\"{}\" name=\"{}\" sourceRef=\"{}\" targetRef=\"{}\"",
                escape_xml_string(&flow.id),
                escape_xml_string(flow.name.as_ref().unwrap_or(&String::new())),
                escape_xml_string(&flow.source_id),
                escape_xml_string(&flow.target_id)
            ));

            if let Some(condition) = &flow.condition {
                xml.push_str(">\n");
                xml.push_str(&format!(
                    "      <bpmn:conditionExpression xsi:type=\"bpmn:tFormalExpression\">{}</bpmn:conditionExpression>\n",
                    condition
                ));
                xml.push_str("    </bpmn:sequenceFlow>\n");
            } else {
                xml.push_str("/>\n");
            }
        }

        xml.push_str("  </bpmn:process>\n");

        // Diagram plane
        xml.push_str("  <bpmndi:BPMNDiagram id=\"BPMNDiagram_1\">\n");
        xml.push_str(&format!(
            "    <bpmndi:BPMNPlane id=\"BPMNPlane_1\" bpmnElement=\"Process_{}\">",
            diagram.id
        ));
        xml.push('\n');

        // Diagram elements (positions can be added later)
        for event in diagram.events.values() {
            xml.push_str(&format!(
                "      <bpmndi:BPMNShape id=\"BPMNShape_{}\" bpmnElement=\"{}\">\n",
                event.id, event.id
            ));
            xml.push_str("        <dc:Bounds x=\"100\" y=\"100\" width=\"36\" height=\"36\"/>\n");
            xml.push_str("      </bpmndi:BPMNShape>\n");
        }

        for task in diagram.tasks.values() {
            xml.push_str(&format!(
                "      <bpmndi:BPMNShape id=\"BPMNShape_{}\" bpmnElement=\"{}\">\n",
                task.id, task.id
            ));
            xml.push_str("        <dc:Bounds x=\"100\" y=\"200\" width=\"100\" height=\"80\"/>\n");
            xml.push_str("      </bpmndi:BPMNShape>\n");
        }

        for gateway in diagram.gateways.values() {
            xml.push_str(&format!(
                "      <bpmndi:BPMNShape id=\"BPMNShape_{}\" bpmnElement=\"{}\">\n",
                gateway.id, gateway.id
            ));
            xml.push_str("        <dc:Bounds x=\"100\" y=\"300\" width=\"50\" height=\"50\"/>\n");
            xml.push_str("      </bpmndi:BPMNShape>\n");
        }

        for flow in diagram.flows.values() {
            xml.push_str(&format!(
                "      <bpmndi:BPMNEdge id=\"BPMNEdge_{}\" bpmnElement=\"{}\">\n",
                flow.id, flow.id
            ));
            xml.push_str("        <di:waypoint x=\"150\" y=\"200\"/>\n");
            xml.push_str("        <di:waypoint x=\"150\" y=\"300\"/>\n");
            xml.push_str("      </bpmndi:BPMNEdge>\n");
        }

        xml.push_str("    </bpmndi:BPMNPlane>\n");
        xml.push_str("  </bpmndi:BPMNDiagram>\n");

        xml.push_str("</bpmn:definitions>\n");

        xml
    }

    /// Parse BPMN diagram from XML string (simplified)
    pub fn from_xml(xml: &str) -> Result<BPMNDiagram, String> {
        // Simple XML parser - in production would use proper XML library
        let mut diagram = BPMNDiagram::new("Imported");

        // Extract process ID
        if let Some(process_start) = xml.find("<bpmn:process") {
            if let Some(id_start) = xml[process_start..].find("id=\"") {
                if let Some(id_end) = xml[process_start + id_start + 4..].find("\"") {
                    let process_id = xml
                        [process_start + id_start + 4..process_start + id_start + 4 + id_end]
                        .to_string();
                    diagram.id = process_id;
                }
            }
        }

        // Parse start events
        for line in xml.lines() {
            if line.contains("<bpmn:startEvent") {
                if let Some(id) = Self::extract_attribute(line, "id") {
                    if let Some(name) = Self::extract_attribute(line, "name") {
                        let event = Event::new(name, EventType::Start);
                        let mut event = event;
                        event.id = id;
                        diagram.add_event(event);
                    }
                }
            }

            if line.contains("<bpmn:endEvent") {
                if let Some(id) = Self::extract_attribute(line, "id") {
                    if let Some(name) = Self::extract_attribute(line, "name") {
                        let event = Event::new(name, EventType::End);
                        let mut event = event;
                        event.id = id;
                        diagram.add_event(event);
                    }
                }
            }

            if line.contains("<bpmn:userTask") {
                if let Some(id) = Self::extract_attribute(line, "id") {
                    if let Some(name) = Self::extract_attribute(line, "name") {
                        let task = Task::new(name, TaskType::UserTask);
                        let mut task = task;
                        task.id = id;
                        diagram.add_task(task);
                    }
                }
            }

            if line.contains("<bpmn:serviceTask") {
                if let Some(id) = Self::extract_attribute(line, "id") {
                    if let Some(name) = Self::extract_attribute(line, "name") {
                        let task = Task::new(name, TaskType::ServiceTask);
                        let mut task = task;
                        task.id = id;
                        diagram.add_task(task);
                    }
                }
            }

            if line.contains("<bpmn:exclusiveGateway") {
                if let Some(id) = Self::extract_attribute(line, "id") {
                    if let Some(name) = Self::extract_attribute(line, "name") {
                        let gateway = Gateway::new(name, GatewayType::ExclusiveXor);
                        let mut gateway = gateway;
                        gateway.id = id;
                        diagram.add_gateway(gateway);
                    }
                }
            }

            if line.contains("<bpmn:parallelGateway") {
                if let Some(id) = Self::extract_attribute(line, "id") {
                    if let Some(name) = Self::extract_attribute(line, "name") {
                        let gateway = Gateway::new(name, GatewayType::Parallel);
                        let mut gateway = gateway;
                        gateway.id = id;
                        diagram.add_gateway(gateway);
                    }
                }
            }

            if line.contains("<bpmn:sequenceFlow") {
                if let Some(id) = Self::extract_attribute(line, "id") {
                    if let Some(source) = Self::extract_attribute(line, "sourceRef") {
                        if let Some(target) = Self::extract_attribute(line, "targetRef") {
                            let flow = SequenceFlow::new(source, target);
                            let mut flow = flow;
                            flow.id = id;
                            if let Some(name) = Self::extract_attribute(line, "name") {
                                flow.name = Some(name);
                            }
                            diagram.add_flow(flow);
                        }
                    }
                }
            }
        }

        diagram.validate()?;
        Ok(diagram)
    }

    /// Extract attribute value from XML element
    fn extract_attribute(element: &str, attr_name: &str) -> Option<String> {
        let pattern = format!("{}=\"", attr_name);
        if let Some(start) = element.find(&pattern) {
            let value_start = start + pattern.len();
            if let Some(end) = element[value_start..].find("\"") {
                return Some(element[value_start..value_start + end].to_string());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xml_export_simple() {
        let mut diagram = BPMNDiagram::new("Test Process");
        let start = Event::new("Start", EventType::Start);
        let end = Event::new("End", EventType::End);
        let task = Task::new("Task", TaskType::UserTask);

        let start_id = diagram.add_event(start);
        let task_id = diagram.add_task(task);
        let end_id = diagram.add_event(end);

        diagram.add_flow(SequenceFlow::new(start_id, task_id.clone()));
        diagram.add_flow(SequenceFlow::new(task_id, end_id));

        let xml = BPMNXmlBuilder::to_xml(&diagram);
        assert!(xml.contains("<bpmn:startEvent"));
        assert!(xml.contains("<bpmn:userTask"));
        assert!(xml.contains("<bpmn:endEvent"));
        assert!(xml.contains("<bpmn:sequenceFlow"));
    }

    #[test]
    fn test_xml_export_with_gateways() {
        let mut diagram = BPMNDiagram::new("Test");
        let start = Event::new("Start", EventType::Start);
        let gateway = Gateway::new("Decision", GatewayType::ExclusiveXor);
        let end = Event::new("End", EventType::End);

        let start_id = diagram.add_event(start);
        let gateway_id = diagram.add_gateway(gateway);
        let end_id = diagram.add_event(end);

        diagram.add_flow(SequenceFlow::new(start_id, gateway_id.clone()));
        diagram.add_flow(SequenceFlow::new(gateway_id, end_id));

        let xml = BPMNXmlBuilder::to_xml(&diagram);
        assert!(xml.contains("<bpmn:exclusiveGateway"));
    }

    #[test]
    fn test_xml_export_with_conditions() {
        let mut diagram = BPMNDiagram::new("Test");
        let start = Event::new("Start", EventType::Start);
        let gateway = Gateway::new("Decision", GatewayType::ExclusiveXor);
        let end = Event::new("End", EventType::End);

        let start_id = diagram.add_event(start);
        let gateway_id = diagram.add_gateway(gateway);
        let end_id = diagram.add_event(end);

        diagram.add_flow(SequenceFlow::new(start_id, gateway_id.clone()));
        let flow = SequenceFlow::new(gateway_id, end_id).with_condition("status == 'approved'");
        diagram.add_flow(flow);

        let xml = BPMNXmlBuilder::to_xml(&diagram);
        assert!(xml.contains("status == 'approved'"));
    }

    #[test]
    fn test_xml_parse_simple() {
        let mut diagram = BPMNDiagram::new("Test Process");
        let start = Event::new("Start", EventType::Start);
        let task = Task::new("DoWork", TaskType::UserTask);
        let end = Event::new("End", EventType::End);

        let start_id = diagram.add_event(start);
        let task_id = diagram.add_task(task);
        let end_id = diagram.add_event(end);

        diagram.add_flow(SequenceFlow::new(start_id, task_id.clone()));
        diagram.add_flow(SequenceFlow::new(task_id, end_id));

        let xml = BPMNXmlBuilder::to_xml(&diagram);
        let parsed = BPMNXmlBuilder::from_xml(&xml);

        assert!(parsed.is_ok());
        let parsed_diagram = parsed.unwrap();
        assert!(parsed_diagram.start_event_id.is_some());
        assert!(!parsed_diagram.end_event_ids.is_empty());
    }

    #[test]
    fn test_xml_round_trip() {
        let mut diagram = BPMNDiagram::new("Round Trip Test");
        let start = Event::new("Start", EventType::Start);
        let task1 = Task::new("Task 1", TaskType::UserTask);
        let task2 = Task::new("Task 2", TaskType::ServiceTask);
        let end = Event::new("End", EventType::End);

        let start_id = diagram.add_event(start);
        let task1_id = diagram.add_task(task1);
        let task2_id = diagram.add_task(task2);
        let end_id = diagram.add_event(end);

        diagram.add_flow(SequenceFlow::new(start_id, task1_id.clone()));
        diagram.add_flow(SequenceFlow::new(task1_id, task2_id.clone()));
        diagram.add_flow(SequenceFlow::new(task2_id, end_id));

        let xml = BPMNXmlBuilder::to_xml(&diagram);
        let parsed = BPMNXmlBuilder::from_xml(&xml).unwrap();

        assert_eq!(parsed.tasks.len(), 2);
        assert!(!parsed.start_event_id.is_none());
        assert!(!parsed.end_event_ids.is_empty());
    }
}
