
public protocol {{ obj.name() }}Protocol {
    {% for meth in obj.methods() -%}
    func {{ meth.name()|fn_name_swift }}({% call swift::arg_list_protocol(meth) %}) {% call swift::throws(meth) -%}
    {%- match meth.return_type() -%}
    {%- when Some with (return_type) %} -> {{ return_type|type_swift -}}
    {%- else -%}
    {%- endmatch %}
    {% endfor %}
}

public class {{ obj.name() }}: {{ obj.name() }}Protocol {
    private let handle: UInt64

    {%- for cons in obj.constructors() %}
    public init({% call swift::arg_list_decl(cons) -%}) {% call swift::throws(cons) %} {
        self.handle = {% call swift::to_ffi_call(cons) %}
    }
    {%- endfor %}

    deinit {
        try! rustCall(InternalError.unknown()) { err in
            {{ obj.ffi_object_free().name() }}(handle, err)
        }
    }

    // TODO: Maybe merge the two templates (i.e the one with a return type and the one without)
    {% for meth in obj.methods() -%}
    {%- match meth.return_type() -%}

    {%- when Some with (return_type) -%}
    public func {{ meth.name()|fn_name_swift }}({% call swift::arg_list_decl(meth) %}) {% call swift::throws(meth) %} -> {{ return_type|type_swift }} {
        let _retval = {% call swift::to_ffi_call_with_prefix("self.handle", meth) %}
        return {% call swift::try(meth) %} {{ "_retval"|lift_swift(return_type) }}
    }

    {%- when None -%}
    public func {{ meth.name()|fn_name_swift }}({% call swift::arg_list_decl(meth) %}) {% call swift::throws(meth) %} {
        {% call swift::to_ffi_call_with_prefix("self.handle", meth) %}
    }
    {%- endmatch %}
    {% endfor %}
}